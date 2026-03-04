export interface HookSettings {
  permission_prompt: boolean;
  idle_prompt: boolean;
  stop: boolean;
}

export interface Settings {
  hooks: HookSettings;
  notification_duration: number;
  play_sound: boolean;
  auto_start: boolean;
  server_port: number;
  suppress_when_focused: boolean;
}
