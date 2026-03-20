/** 与后端 `ReminderConfig`（camelCase）一致 */
export interface ReminderConfig {
  enableHourly: boolean;
  enableBreak: boolean;
  enableLongWork: boolean;
  breakIntervalMinutes: number;
  longWorkMinutes: number;
}

export const DEFAULT_REMINDER_CONFIG: ReminderConfig = {
  enableHourly: true,
  enableBreak: true,
  enableLongWork: true,
  breakIntervalMinutes: 60,
  longWorkMinutes: 120,
};
