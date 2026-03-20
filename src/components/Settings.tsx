import { useEffect, useRef, useState } from "react";
import { AppConfig } from "../types/config";
import type { ReminderConfig } from "../types/reminderConfig";
import { DEFAULT_REMINDER_CONFIG } from "../types/reminderConfig";
import {
  getConfig,
  getReminderConfig,
  saveConfig,
  testConnectionWith,
  updateReminderConfig,
} from "../utils/tauri";
import "./Settings.css";

type StatusType = "success" | "error" | "loading" | null;

interface Props {
  onClose: () => void;
}

export default function Settings({ onClose }: Props) {
  // ── 表单字段 ─────────────────────────────────────────────────────────────
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("https://api.openai.com/v1");
  const [modelName, setModelName] = useState("gpt-3.5-turbo");
  const [providerType, setProviderType] = useState("openai");

  // ── UI 状态（三态分离，互不影响） ─────────────────────────────────────────
  const [isLoading, setIsLoading] = useState(true);   // 初始配置加载
  const [isTesting, setIsTesting] = useState(false);  // 测试连接中
  const [isSaving, setIsSaving] = useState(false);    // 保存中
  const [isReminderSaving, setIsReminderSaving] = useState(false);

  const [reminderConfig, setReminderConfig] = useState<ReminderConfig>(
    DEFAULT_REMINDER_CONFIG,
  );

  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [statusType, setStatusType] = useState<StatusType>(null);

  // 自动关闭定时器的 ref，防止内存泄漏
  const closeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // 任意操作进行中时禁用主表单按钮
  const isBusy = isLoading || isTesting || isSaving;
  const reminderFieldsLocked = isLoading || isBusy || isReminderSaving;

  // ── 初始化：挂载时加载配置 ─────────────────────────────────────────────
  useEffect(() => {
    Promise.all([
      getConfig(),
      getReminderConfig().catch(() => DEFAULT_REMINDER_CONFIG),
    ])
      .then(([cfg, rem]) => {
        setBaseUrl(cfg.base_url);
        setModelName(cfg.model_name);
        setProviderType(cfg.provider_type);
        setReminderConfig(rem);
        // api_key 后端已脱敏为 "****"，不回填，保持空白提示用户重新输入
      })
      .catch((e) => showStatus(`加载配置失败: ${String(e)}`, "error"))
      .finally(() => setIsLoading(false));

    // 卸载时清除定时器
    return () => {
      if (closeTimerRef.current) clearTimeout(closeTimerRef.current);
    };
  }, []);

  // ── 操作处理 ──────────────────────────────────────────────────────────────

  /**
   * 测试连接：用表单当前值发起测试，不保存、不修改任何配置。
   * 这样改了 URL / Key 后不需要先保存就能验证是否可用。
   */
  async function handleTestConnection() {
    setIsTesting(true);
    showStatus("正在测试连接...", "loading");

    try {
      const ok = await testConnectionWith(baseUrl, apiKey);
      showStatus(
        ok ? "✅ 连接成功！" : "❌ 连接失败：服务不可达，请检查 URL 和 Key",
        ok ? "success" : "error"
      );
    } catch (e) {
      showStatus(`❌ 连接失败：${String(e)}`, "error");
    } finally {
      setIsTesting(false);
    }
  }

  /** 保存配置：写盘成功后 1.2s 自动关闭面板 */
  async function handleSave() {
    setIsSaving(true);
    showStatus("正在保存...", "loading");

    try {
      await saveConfig(buildConfig());
      showStatus("✅ 配置已保存", "success");
      // 短暂显示成功提示后自动关闭
      closeTimerRef.current = setTimeout(onClose, 1200);
    } catch (e) {
      showStatus(`❌ 保存失败：${String(e)}`, "error");
      setIsSaving(false);
    }
    // 保存成功后不重置 isSaving，防止关闭前按钮闪烁
  }

  async function handleSaveReminders() {
    setIsReminderSaving(true);
    showStatus("正在保存提醒设置...", "loading");
    try {
      const breakM = Math.max(1, Math.floor(reminderConfig.breakIntervalMinutes));
      const longM = Math.max(1, Math.floor(reminderConfig.longWorkMinutes));
      await updateReminderConfig({
        ...reminderConfig,
        breakIntervalMinutes: breakM,
        longWorkMinutes: longM,
      });
      setReminderConfig((c) => ({
        ...c,
        breakIntervalMinutes: breakM,
        longWorkMinutes: longM,
      }));
      showStatus("✅ 提醒设置已保存", "success");
    } catch (e) {
      showStatus(`❌ 保存提醒失败：${String(e)}`, "error");
    } finally {
      setIsReminderSaving(false);
    }
  }

  // ── 工具函数 ──────────────────────────────────────────────────────────────

  function buildConfig(): AppConfig {
    return {
      api_key: apiKey,       // 空字符串时后端会保留原密钥
      base_url: baseUrl,
      model_name: modelName,
      provider_type: providerType,
    };
  }

  function showStatus(msg: string, type: StatusType) {
    setStatusMessage(msg);
    setStatusType(type);
  }

  // ── 渲染 ──────────────────────────────────────────────────────────────────

  return (
    <div className="settings-overlay">
      <div className="settings-panel">
        {/* 标题栏 */}
        <div className="settings-header">
          <span className="settings-title">Moocha 设置</span>
          <button
            className="settings-close"
            onClick={onClose}
            title="关闭"
            disabled={isBusy}
          >
            ✕
          </button>
        </div>

        <div className="settings-divider" />

        {/* 初始加载占位 */}
        {isLoading ? (
          <div className="status-message loading">加载配置中...</div>
        ) : (
          <div className="settings-form">
            <div className="form-field">
              <label className="form-label">服务商</label>
              <input
                className="form-input"
                value={providerType}
                onChange={(e) => setProviderType(e.target.value)}
                placeholder="openai / claude / ollama"
                disabled={isBusy}
              />
            </div>

            <div className="form-field">
              <label className="form-label">Base URL</label>
              <input
                className="form-input"
                value={baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                placeholder="https://api.openai.com/v1"
                disabled={isBusy}
              />
            </div>

            <div className="form-field">
              <label className="form-label">模型</label>
              <input
                className="form-input"
                value={modelName}
                onChange={(e) => setModelName(e.target.value)}
                placeholder="gpt-3.5-turbo"
                disabled={isBusy}
              />
            </div>

            <div className="form-field">
              <label className="form-label">API Key</label>
              <input
                className="form-input"
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="留空则保留已保存的 Key"
                disabled={isBusy}
                autoComplete="off"
              />
            </div>

            {/* 操作按钮 */}
            <div className="settings-actions">
              <button
                className="btn btn-test"
                onClick={handleTestConnection}
                disabled={isBusy}
              >
                {isTesting ? "测试中..." : "测试连接"}
              </button>
              <button
                className="btn btn-save"
                onClick={handleSave}
                disabled={isBusy}
              >
                {isSaving ? "保存中..." : "保存"}
              </button>
            </div>

            <div className="settings-divider" />

            <div className="reminder-section-title">休息与定时提醒</div>

            <div className="reminder-toggle-row">
              <span className="reminder-toggle-label">整点报时</span>
              <input
                type="checkbox"
                className="reminder-toggle"
                checked={reminderConfig.enableHourly}
                onChange={(e) =>
                  setReminderConfig((c) => ({
                    ...c,
                    enableHourly: e.target.checked,
                  }))
                }
                disabled={reminderFieldsLocked}
              />
            </div>
            <div className="reminder-toggle-row">
              <span className="reminder-toggle-label">按间隔休息提醒</span>
              <input
                type="checkbox"
                className="reminder-toggle"
                checked={reminderConfig.enableBreak}
                onChange={(e) =>
                  setReminderConfig((c) => ({
                    ...c,
                    enableBreak: e.target.checked,
                  }))
                }
                disabled={reminderFieldsLocked}
              />
            </div>
            <div className="reminder-toggle-row">
              <span className="reminder-toggle-label">久坐提醒</span>
              <input
                type="checkbox"
                className="reminder-toggle"
                checked={reminderConfig.enableLongWork}
                onChange={(e) =>
                  setReminderConfig((c) => ({
                    ...c,
                    enableLongWork: e.target.checked,
                  }))
                }
                disabled={reminderFieldsLocked}
              />
            </div>

            <div className="form-field">
              <label className="form-label">休息间隔（分钟）</label>
              <input
                className="form-input form-input-number"
                type="number"
                min={1}
                max={480}
                value={reminderConfig.breakIntervalMinutes}
                onChange={(e) => {
                  const v = parseInt(e.target.value, 10);
                  setReminderConfig((c) => ({
                    ...c,
                    breakIntervalMinutes: Number.isFinite(v) ? v : c.breakIntervalMinutes,
                  }));
                }}
                disabled={reminderFieldsLocked}
              />
            </div>
            <div className="form-field">
              <label className="form-label">久坐判定（连续工作分钟）</label>
              <input
                className="form-input form-input-number"
                type="number"
                min={1}
                max={720}
                value={reminderConfig.longWorkMinutes}
                onChange={(e) => {
                  const v = parseInt(e.target.value, 10);
                  setReminderConfig((c) => ({
                    ...c,
                    longWorkMinutes: Number.isFinite(v) ? v : c.longWorkMinutes,
                  }));
                }}
                disabled={reminderFieldsLocked}
              />
            </div>

            <button
              type="button"
              className="btn-reminder-save"
              onClick={() => void handleSaveReminders()}
              disabled={reminderFieldsLocked}
            >
              {isReminderSaving ? "保存中..." : "保存提醒设置"}
            </button>

            {/* 状态提示（带淡入动画） */}
            {statusMessage && (
              <div
                key={statusMessage}
                className={`status-message ${statusType ?? ""}`}
              >
                {statusMessage}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
