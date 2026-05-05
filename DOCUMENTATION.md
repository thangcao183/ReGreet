# ReGreet Developer Documentation

## Tổng quan
ReGreet là một greeter GTK4 cho greetd, viết bằng Rust với Relm4. Ứng dụng chạy trên Wayland và cho phép chọn user, session, nhập thủ công user/session, xử lý xác thực tương tác, và khởi động session qua greetd.

## Kiến trúc chính

### Entry point
- `src/main.rs`
  - Phân tích `Args` bằng `clap`.
  - Khởi tạo logging qua `init_logging`.
  - Tạo `RelmApp` và khởi chạy component chính `Greeter` với `GreeterInit`.

### Các module chính
- `src/config.rs` - nạp cấu hình TOML, giá trị mặc định, getter cho appearance, GTK, background, và command.
- `src/constants.rs` - hằng số compile-time, đường dẫn mặc định, và compile-time environment variables.
- `src/tomlutils.rs` - hàm chung `load_toml` dùng serde để đọc file TOML an toàn, trả về default nếu file bị lỗi hoặc thiếu.
- `src/cache/mod.rs` - lưu trữ cache giữa các lần login (last user, session cuối cùng của từng user).
- `src/cache/lru.rs` - wrapper serde cho `lru::LruCache`.
- `src/client.rs` - giao tiếp với `greetd` qua `greetd_ipc`, xử lý demo mode, tạo/cancel session, gửi auth response, và start session.
- `src/sysutil.rs` - thu thập user từ AccountsService và session từ `.desktop` files.
- `src/gui` - giao diện người dùng với Relm4.
  - `component.rs` - cấu hình widget, kết nối signal, cập nhật GUI.
  - `model.rs` - logic greeter, trạng thái, xử lý login và khởi động session.
  - `messages.rs` - dữ liệu và enum giao tiếp giữa view và model.
  - `templates.rs` - định nghĩa UI bằng Relm4 templates.
  - `widget/clock.rs` - widget đồng hồ cấu hình được.

## Chi tiết từng module

### `src/main.rs`
- `LogLevel` enum: `Off`, `Error`, `Warn`, `Info`, `Debug`, `Trace`.
- `Args`:
  - `logs`: đường dẫn file log.
  - `log_level`: mức log.
  - `verbose`: ghi log ra stdout.
  - `config`: đường dẫn config TOML.
  - `style`: đường dẫn CSS.
  - `demo`: bật demo mode.
- `setup_log_file` tạo file log và thư mục nếu cần.
- `init_logging` thiết lập `tracing_subscriber` với file rotation, và log panic.

### `src/constants.rs`
- Macro `env_or!` lấy giá trị env const hoặc default.
- `APP_ID`: `apps.regreet`.
- Đường dẫn mặc định:
  - `CONFIG_PATH`: `/etc/greetd/regreet.toml`
  - `CSS_PATH`: `/etc/greetd/regreet.css`
  - `CACHE_PATH`: `/var/lib/regreet/state.toml`
  - `LOG_PATH`: `/var/log/regreet/log`
- Các lệnh mặc định:
  - `REBOOT_CMD`: `reboot`
  - `POWEROFF_CMD`: `poweroff`
  - `GREETING_MSG`: `Welcome back!`
  - `SESSION_DIRS`: `/usr/share/xsessions:/usr/share/wayland-sessions`
  - `X11_CMD_PREFIX`: `startx /usr/bin/env`

### `src/tomlutils.rs`
- `TomlFileError` bao gồm I/O, UTF-8, decode/encode TOML.
- `load_raw_toml` đọc file và parse.
- `load_toml`:
  - nếu file tồn tại: nạp và log.
  - nếu file không tồn tại hoặc lỗi parse: warn và trả về `Default::default()`.

### `src/config.rs`
- `AppearanceSettings`:
  - `greeting_msg`.
- `GtkSettings`:
  - `application_prefer_dark_theme`
  - `cursor_theme_name`
  - `cursor_blink`
  - `font_name`
  - `icon_theme_name`
  - `theme_name`
- `BgFit` enum cho background fit (`Fill`, `Contain`, `Cover`, `ScaleDown`).
- `Background` struct: `path`, `fit`.
- `SystemCommands` struct:
  - `reboot`
  - `poweroff`
  - `x11_prefix`
- `Config`:
  - `appearance`
  - `env`
  - `background`
  - `gtk`
  - `commands`
  - `widget`
  - supports video backgrounds when `background.path` points to a supported video file.
- Phương thức `Config::new(path)` đọc TOML.
- Getter: `get_env`, `get_background`, `get_background_fit`, `get_gtk_settings`, `get_sys_commands`, `get_default_message`.

### `src/cache/mod.rs`
- `Cache`:
  - `last_user: Option<String>`
  - `user_to_last_sess: LruCache<String, String>`
- `Cache::new()` đọc `CACHE_PATH` với `load_toml`.
- `save()` lưu lại file `state.toml`, tạo thư mục nếu cần.
- getter/setter cho last user/session.
- `CACHE_LIMIT = 100`.

### `src/cache/lru.rs`
- Wrapper `LruCache<K, V>` để hỗ trợ serde.
- Implement `Deref`, `DerefMut` về `OrigLruCache`.
- `Deserialize` tuần tự đọc map và push vào LRU.
- `Serialize` xuất map entries.

### `src/client.rs`
- `GREETD_SOCK_ENV_VAR = "GREETD_SOCK"`
- `DEMO_*` constants cho chế độ demo:
  - OTP `0248`
  - password `pass`
- `AuthStatus`: `NotStarted`, `InProgress`, `Done`.
- `GreetdClient`:
  - `socket: Option<UnixStream>`
  - `auth_status`
- `new(demo)`:
  - nếu demo: không kết nối socket.
  - nếu không: kết nối GREETD_SOCK.
- `create_session(username)`:
  - gửi `Request::CreateSession`.
  - nếu demo: trả `AuthMessage(Secret, "One-Time Password:")`.
  - cập nhật `auth_status`.
- `send_auth_response(input)`:
  - gửi `Request::PostAuthMessageResponse`.
  - demo mode: OTP/Password logic.
  - cập nhật `auth_status`.
- `start_session(command, environment)`:
  - gửi `Request::StartSession`.
- `cancel_session()`:
  - gửi `Request::CancelSession`.
- `get_auth_status()`.

### `src/sysutil.rs`
- `SessionType`: `X11`, `Wayland`, `Unknown`.
- `SessionInfo`: `command: Vec<String>`, `sess_type: SessionType`.
- `SysUtil` lưu:
  - `users: HashMap<full name, username>`
  - `shells: HashMap<username, shell>`
  - `sessions: HashMap<session name, SessionInfo>`
- `SysUtil::new(config)`:
  - mở kết nối system D-Bus.
  - dùng AccountsServiceProxy để liệt kê và đọc user.
  - xây dựng `users` và `shells`.
  - gọi `init_sessions(config)`.
- `init_sessions(config)`:
  - lấy `SESSION_DIRS`, hoặc dùng `XDG_DATA_DIRS` nếu có.
  - duyệt từng thư mục `xsessions` và `wayland-sessions`.
  - đọc `.desktop`, parse `Exec=`, `Name=`, `Hidden=`, `NoDisplay=`.
  - áp dụng `x11_prefix` cho session X11.
  - bỏ qua session ẩn và duplicate theo đường dẫn.
  - lưu `command` và `sess_type`.
- Getter: `get_users`, `get_shells`, `get_sessions`.

### `src/gui/templates.rs`
- Template UI gồm:
  - `gtk::Overlay` với background `gtk::Picture` hoặc `gtk::Video`.
  - Frame trung tâm chứa box user/session/login.
  - `ComboBoxText` cho user và session.
  - `Entry` cho manual user/session.
  - `PasswordEntry` và `Entry` cho auth input.
  - Toggle button cho manual user/session.
  - `Login`, `Cancel`, `Reboot`, `Power Off` buttons.
  - InfoBar hiển thị lỗi.
  - `Clock` widget.

### `src/gui/messages.rs`
- `UserSessInfo` chứa trạng thái user/session UI hiện tại.
- `InputMsg` messages từ view đến model:
  - `Login { input, info }`
  - `Cancel`
  - `UserChanged(UserSessInfo)`
  - `ToggleManualUser`
  - `ToggleManualSess`
  - `Reboot`
  - `PowerOff`
- `CommandMsg` messages nội bộ/background:
  - `ClearErr`
  - `HandleGreetdResponse(Response)`
  - `MonitorRemoved(GString)`

### `src/gui/component.rs`
- `GreeterInit` chứa `config_path`, `css_path`, `demo`.
- `setup_settings` áp dụng `GtkSettings` vào `gtk::Settings`.
- `setup_users_sessions` nạp user/session combo box và chọn user mặc định.
- Tạo Relm4 async component `Greeter`.
- View bindings dựa trên `Updates` tracker:
  - ẩn/hiện widget theo `manual_user_mode`, `manual_sess_mode`, `input_mode`.
  - connect events tới `InputMsg`.
- `post_view` fullscreen window khi monitor cập nhật.
- `init`:
  - tạo model `Greeter::new`.
  - cancel session hiện tại nếu có.
  - chọn monitor, fullscreen, áp dụng GTK settings.
  - nạp CSS tùy chỉnh nếu tồn tại.
  - đặt default widget là login button.
- `update` xử lý InputMsg:
  - `Login` -> `login_click_handler`
  - `Cancel` -> `cancel_click_handler`
  - `UserChanged` -> `user_change_handler`
  - toggle manual modes
  - reboot/poweroff
- `update_cmd` xử lý background command messages.

### `src/gui/model.rs`
- `InputMode`: `None`, `Secret`, `Visible`.
- `Updates` tracker fields:
  - `message`, `error`, `input`, `manual_user_mode`, `manual_sess_mode`, `input_prompt`, `input_mode`, `active_session_id`, `time`, `monitor`.
- `Greeter` model giữ:
  - `greetd_client`, `sys_util`, `cache`, `config`, `sess_info`, `updates`, `demo`, `clock`.
- `new(config_path, demo)`:
  - đọc config.
  - khởi tạo `GreetdClient`.
  - tạo `Clock` widget.
  - nạp `SysUtil` và `Cache`.
- `choose_monitor` chọn monitor đầu tiên và cài callback monitor remove.
- `run_cmd` thực thi command hệ thống bất đồng bộ.
- `reboot_click_handler`/`poweroff_click_handler`: dùng `commands` config và không làm gì nếu demo.
- `cancel_click_handler`: cancel session, reset input state.
- `create_session`:
  - kiểm tra manual session command.
  - gọi `greetd_client.create_session`.
  - xử lý response qua `handle_greetd_response`.
- `handle_greetd_response`:
  - `Success`: `start_session`
  - `AuthMessage` secret/visible/info/error xử lý giao diện và gửi response rỗng nếu cần.
  - `Error`: show error và cancel nếu auth error.
- `user_change_handler` chọn session gần nhất của user nếu đã cache.
- `login_click_handler` tuỳ theo `AuthStatus`: bắt đầu session, gửi input, hoặc bắt đầu tạo session.
- `send_input` gửi auth response từ người dùng.
- `get_current_username` lấy user hiện tại từ `UserSessInfo`.
- `get_current_session_info` trả về session command theo manual hoặc từ session list; nếu không có session thì dùng shell login.
- `start_session`:
  - tạo env `XDG_SESSION_TYPE` + env từ config.
  - lưu cache last user/session.
  - gọi `greetd_client.start_session`.
  - nếu thành công thì `std::process::exit(0)`.
- `display_error` hiển thị error message và đặt timer để xoá sau 5s.
- `Drop` tự động cancel session khi thoát.

### `src/gui/widget/clock.rs`
- `ClockConfig`:
  - `format`, `resolution`, `timezone`, `label_width`.
- `parse_tz` chuyển timezone string; invalid thì dùng system timezone.
- `Clock` component:
  - gửi `Tick` định kỳ.
  - khi nhận `Tick`, cập nhật `current_time` theo format và timezone.

## Cấu hình và tùy chỉnh

### File cấu hình mẫu
- `regreet.sample.toml` trình bày đầy đủ các trường hiện có:
  - `[background]`: `path`, `fit`.
  - `[env]`: biến môi trường cho session.
  - `[GTK]`: dark theme, cursor, font, icon theme, GTK theme.
  - `[commands]`: `reboot`, `poweroff`, `x11_prefix`.
  - `[appearance]`: `greeting_msg`.
  - `[widget.clock]`: `format`, `resolution`, `timezone`, `label_width`.

### Cấu hình tại runtime
- `--config PATH` để dùng config riêng.
- `--style PATH` để nạp CSS riêng.
- `--demo` bật chế độ demo mà không cần greetd.

### Biến môi trường compile-time và runtime
- `GREETD_CONFIG_DIR`: thư mục cấu hình `greetd`.
- `STATE_DIR`: thư mục lưu cache.
- `LOG_DIR`: thư mục log.
- `SESSION_DIRS`: danh sách thư mục chứa session `.desktop`.
- `X11_CMD_PREFIX`: prefix khi khởi session X11.
- `REBOOT_CMD`, `POWEROFF_CMD`: command mặc định nếu không cấu hình.

## Flow đăng nhập
1. Greeter khởi tạo, nạp config, users, sessions, cache.
2. UI hiển thị user/session, background, clock.
3. Người dùng chọn user/session hoặc nhập thủ công.
4. Nhấn Login:
   - nếu chưa có auth session: `create_session`
   - nếu `greetd` trả `AuthMessage` yêu cầu input: hiển thị prompt và chờ input.
   - khi có input: `send_auth_response`.
   - nếu `Success`: `start_session`.
5. `start_session` truyền `XDG_SESSION_TYPE` và env config vào greetd.
6. Nếu thành công, ứng dụng thoát và greetd bắt đầu session mới.

## Chức năng chính
- Tự động phát hiện user qua AccountsService.
- Tự động phát hiện session X11/Wayland từ `.desktop`.
- Hỗ trợ `Hidden=true` và `NoDisplay=true` trong file session.
- Hỗ trợ manual entry username và session command.
- Lưu user/session cuối cùng vào cache.
- Cho phép load CSS tùy chỉnh.
- Cho phép cấu hình GTK theme, font, cursor, dark mode.
- Cho phép cấu hình background và cách fit ảnh.
- Hỗ trợ reboot và shutdown bằng command tuỳ chỉnh.
- Hỗ trợ demo mode không cần greetd.

## Các điểm dễ mở rộng

### Thêm field cấu hình mới
1. Thêm trường vào `Config` ở `src/config.rs`.
2. Cập nhật `regreet.sample.toml`.
3. Nếu cần, thêm getter phù hợp.
4. Sử dụng giá trị cấu hình trong `src/gui/component.rs` hoặc `src/gui/model.rs`.

### Mở rộng GUI
1. Thêm widget vào `src/gui/templates.rs`.
2. Định nghĩa `InputMsg` mới nếu widget gửi action.
3. Bắt message mới trong `src/gui/component.rs::update`.
4. Xử lý logic trong `src/gui/model.rs`.

### Thêm hành động hệ thống (như Sleep)
1. Thêm `InputMsg::Sleep` hoặc tương tự.
2. Thêm button UI trong template.
3. Bổ sung handler trong `model.rs`.
4. Nếu cần cấu hình, thêm vào `SystemCommands` và `regreet.sample.toml`.

### Hỗ trợ session nguồn mới
- Cập nhật `SESSION_DIRS` mặc định trong `src/constants.rs` hoặc dùng env `SESSION_DIRS`.
- Nếu cần parse thêm metadata từ `.desktop`, mở rộng `SysUtil::init_sessions` với Regex/desktop parser.

### Hỗ trợ xác thực mới
- Phần lớn flow auth tập trung trong `src/client.rs` và `src/gui/model.rs`.
- `GreetdClient` là nơi tương tác trực tiếp với `greetd`.
- `handle_greetd_response` có thể mở rộng để xử lý các loại `AuthMessageType` khác hoặc custom response.

## Build & debug
- Biên dịch:
  - `cargo build --release`
  - `cargo build --all-features --release` để bật `gtk4_8`.
  - `cargo build -F gtk4_8 --release` để bật support GTK 4.8.
- Chạy thử cấu hình:
  - `cargo run -- --config /etc/greetd/regreet.toml --style /etc/greetd/regreet.css`
  - `cargo run -- --demo`

## Lưu ý quan trọng
- Nếu cấu hình TOML bị lỗi hoặc thiếu, app vẫn chạy với giá trị mặc định.
- Nếu `GREETD_SOCK` không tồn tại và không chạy `--demo`, app sẽ panic tại `GreetdClient::new`.
- `Cache::save()` chỉ ghi khi không phải demo.
- `Drop` của `Greeter` cố cancel session khi thoát.
- `SysUtil::init_sessions` chỉ chọn session đầu tiên nếu tên trùng và cùng loại.

## Kết luận
Tài liệu này tập trung vào:
- hiểu kiến trúc ReGreet,
- các dòng chính của luồng đăng nhập,
- các module dễ mở rộng,
- và cách thêm tính năng mới.

Sử dụng `DOCUMENTATION.md` này làm nền tảng để mở rộng ReGreet trong tương lai.