import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { svgAppearance } from "../config/petAppearance";
import { useMousePosition } from "../hooks/useMousePosition";
import { useRandomBehavior } from "../hooks/useRandomBehavior";
import type { PetBehavior } from "../hooks/useRandomBehavior";
import { useMoodChange } from "../hooks/useMoodChange";
import { useContextActions } from "../hooks/useContextActions";
import type { PetAppearance, PetMood } from "../types/pet";
import type { PetReminderPayload } from "../types/petReminder";
import { acknowledgeReminder, clearPetAlertTop } from "../utils/tauri";
import "./Pet.css";

export type { PetMood };

// ── 视线 & 头部追踪参数 ──────────────────────────────────────────────────────
/** 视线最大偏移量（SVG / CSS 像素） */
const MAX_GAZE_PX  = 4;
/** 头部最大倾斜角度（度） */
const MAX_TILT_DEG = 6;

/** 将 value 限制在 [min, max] */
const clamp = (v: number, min: number, max: number) =>
  Math.max(min, Math.min(max, v));

const PET_MOODS: readonly PetMood[] = ["idle", "happy", "sleepy", "excited", "curious"];

function isPetMood(v: string): v is PetMood {
  return (PET_MOODS as readonly string[]).includes(v);
}

function reminderMoodForKind(kind: string): PetMood | null {
  if (kind === "hourly") return "happy";
  if (kind === "break") return "curious";
  if (kind === "long_work") return "sleepy";
  return null;
}

function reminderBubbleClass(kind: string): string {
  if (kind === "hourly") return "pet-reminder-hourly";
  if (kind === "long_work") return "pet-reminder-long-work";
  return "pet-reminder-break";
}

interface Props {
  mood?: PetMood;
  /** 形象配置；替换形象时只需传入新配置，交互逻辑不受影响 */
  appearance?: PetAppearance;
  /** 情绪变更回调；点击/交互时通知父组件切换情绪 */
  onMoodChange?: (mood: PetMood) => void;
}

/**
 * 宠物展示组件。
 *
 * 表情切换完全由 CSS 类驱动：
 *   根容器附加 `{containerClass}--{mood}` 修饰符
 *   → CSS 规则显示对应的 `.expression-*` 分组
 *   → 无需 React 条件渲染，替换形象不影响此逻辑
 *
 * 视线追踪（Step 2）预留接口：
 *   `.pet-eyes` 和 `.pet-head` 通过 CSS 自定义属性
 *   `--gaze-x / --gaze-y / --head-tilt` 接受 transform 指令
 */
export default function Pet({ mood = "idle", appearance = svgAppearance, onMoodChange }: Props) {
  const { containerClass, expressions } = appearance;

  // 点击动画状态
  const [isClicking, setIsClicking] = useState(false);
  // 用 ref 存储定时器，防止重复点击时定时器堆叠
  const clickTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // 随机行为状态（眨眼 / 伸懒腰 / 打哈欠）
  const [currentBehavior, setCurrentBehavior] = useState<PetBehavior | null>(null);

  /** 行为动画持续时间（与 CSS 保持一致） */
  const BEHAVIOR_DURATION: Record<PetBehavior, number> = {
    blink:   200,
    stretch: 1000,
    yawn:    1500,
  };

  // 随机行为 Hook
  useRandomBehavior((behavior) => {
    setCurrentBehavior(behavior);
    setTimeout(() => setCurrentBehavior(null), BEHAVIOR_DURATION[behavior]);
  });

  // 自然情绪变化 Hook（仅在 idle/happy/sleepy 间轮换）
  useMoodChange(mood, onMoodChange ?? (() => {}));

  /** 情境规则：后端 `context-action` → 切换情绪 + 短暂气泡 */
  const [contextHint, setContextHint] = useState<string | null>(null);
  const contextHintTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const onMoodChangeRef = useRef(onMoodChange);
  onMoodChangeRef.current = onMoodChange;

  useContextActions((batch) => {
    const last = batch[batch.length - 1];
    if (!last) return;
    if (isPetMood(last.mood)) {
      onMoodChangeRef.current?.(last.mood);
    }
    if (contextHintTimerRef.current) clearTimeout(contextHintTimerRef.current);
    setContextHint(last.message);
    contextHintTimerRef.current = setTimeout(() => {
      setContextHint(null);
      contextHintTimerRef.current = null;
    }, 3000);
  });

  /** 定时提醒队列（需用户点「知道了」；与情境气泡分离） */
  const [reminderQueue, setReminderQueue] = useState<PetReminderPayload[]>([]);
  const activeReminder = reminderQueue[0] ?? null;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<PetReminderPayload[]>("pet-reminder", (event) => {
      const batch = event.payload;
      if (!Array.isArray(batch) || batch.length === 0) return;
      setReminderQueue((q) => [...q, ...batch]);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  // 有强提醒时清掉短时情境气泡，避免叠字
  useEffect(() => {
    if (reminderQueue.length > 0) {
      if (contextHintTimerRef.current) clearTimeout(contextHintTimerRef.current);
      contextHintTimerRef.current = null;
      setContextHint(null);
    }
  }, [reminderQueue.length]);

  // 当前提醒对应情绪
  useEffect(() => {
    if (!activeReminder) return;
    const m = reminderMoodForKind(activeReminder.kind);
    if (m) onMoodChangeRef.current?.(m);
  }, [activeReminder]);

  async function dismissActiveReminder() {
    if (!activeReminder) return;
    try {
      await acknowledgeReminder(activeReminder.kind);
    } catch {
      /* 离线/调试时仍可关闭 UI */
    }
    setReminderQueue((q) => {
      const next = q.slice(1);
      if (next.length === 0) {
        void clearPetAlertTop().catch(() => {
          /* 调试或未就绪时忽略 */
        });
      }
      return next;
    });
  }

  useEffect(() => {
    return () => {
      if (contextHintTimerRef.current) clearTimeout(contextHintTimerRef.current);
    };
  }, []);

  // 根容器类：基础类 + 情绪修饰符 + 当前表情类 + 点击动画类 + 行为类
  const rootClass = [
    containerClass,
    `${containerClass}--${mood}`,
    expressions[mood],
    isClicking                ? `${containerClass}--clicking`              : "",
    currentBehavior !== null  ? `${containerClass}--${currentBehavior}`    : "",
  ].filter(Boolean).join(" ");

  /** 单击：播放点击动画，切换到 excited，2 秒后恢复 idle */
  const handleClick = () => {
    if (clickTimerRef.current) clearTimeout(clickTimerRef.current);

    setIsClicking(true);
    onMoodChange?.("excited");

    clickTimerRef.current = setTimeout(() => {
      setIsClicking(false);
      onMoodChange?.("idle");
    }, 2000);
  };

  /** 双击：启动窗口拖拽 */
  const handleDoubleClick = async () => {
    try {
      await getCurrentWindow().startDragging();
    } catch (e) {
      console.error("startDragging failed:", e);
    }
  };

  // SVG ref：用于设置 CSS 自定义属性，驱动视线 & 头部 transform
  const svgRef = useRef<SVGSVGElement>(null);

  // 鼠标位置追踪
  const { position, inWindow } = useMousePosition();

  useEffect(() => {
    const svg = svgRef.current;
    if (!svg) return;

    if (!inWindow) {
      // 鼠标离开窗口 → 平滑归位
      svg.style.setProperty("--gaze-x",    "0px");
      svg.style.setProperty("--gaze-y",    "0px");
      svg.style.setProperty("--head-tilt", "0deg");
      return;
    }

    // 计算鼠标相对于窗口中心的偏移
    const cx = window.innerWidth  / 2;
    const cy = window.innerHeight / 2;
    const dx = position.x - cx;
    const dy = position.y - cy;

    // 归一化：以窗口半宽/半高为基准，映射到各自的最大输出范围
    const gazeX  = clamp((dx / cx) * MAX_GAZE_PX,  -MAX_GAZE_PX,  MAX_GAZE_PX);
    const gazeY  = clamp((dy / cy) * MAX_GAZE_PX,  -MAX_GAZE_PX,  MAX_GAZE_PX);
    const tilt   = clamp((dx / cx) * MAX_TILT_DEG, -MAX_TILT_DEG, MAX_TILT_DEG);

    svg.style.setProperty("--gaze-x",    `${gazeX.toFixed(2)}px`);
    svg.style.setProperty("--gaze-y",    `${gazeY.toFixed(2)}px`);
    svg.style.setProperty("--head-tilt", `${tilt.toFixed(2)}deg`);
  }, [position, inWindow]);

  return (
    <div
      className={rootClass}
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
    >
      {activeReminder ? (
        <div
          className={`pet-reminder-bubble ${reminderBubbleClass(activeReminder.kind)}`}
          role="alertdialog"
          aria-labelledby="pet-reminder-msg"
          onClick={(e) => e.stopPropagation()}
        >
          <p id="pet-reminder-msg">{activeReminder.message}</p>
          <button
            type="button"
            className="pet-reminder-dismiss"
            onClick={(e) => {
              e.stopPropagation();
              void dismissActiveReminder();
            }}
          >
            知道了
          </button>
        </div>
      ) : (
        contextHint && (
          <div className="pet-context-hint" role="status" aria-live="polite">
            {contextHint}
          </div>
        )
      )}
      <svg
        ref={svgRef}
        viewBox="0 0 200 240"
        xmlns="http://www.w3.org/2000/svg"
        className="pet-svg"
        aria-label={`Moocha is ${mood}`}
      >
        {/* ── 身体 & 尾巴（最底层）── */}
        <g className="pet-body">
          <path
            d="M 150,202 C 176,186 184,158 170,132 C 158,110 145,124 152,138"
            fill="#f2d48c" stroke="#3a2200" strokeWidth="2.5" strokeLinecap="round"
          />
          <ellipse cx="100" cy="192" rx="52" ry="46"
            fill="#f2d48c" stroke="#3a2200" strokeWidth="2.5"/>
        </g>

        {/* ── 头部（Step 2 头部跟随的 transform 目标）── */}
        <g className="pet-head">

          {/* 头圆 */}
          <circle cx="100" cy="100" r="56"
            fill="#f2d48c" stroke="#3a2200" strokeWidth="2.5"/>

          {/* 左耳 */}
          <polygon points="44,74 66,12 94,66"
            fill="#f2d48c" stroke="#3a2200" strokeWidth="2.5" strokeLinejoin="round"/>
          <polygon points="53,70 68,30 90,64" fill="#ffaab8"/>
          <path d="M 63,13 L 58,4 M 66,12 L 66,3 M 69,13 L 73,5"
            stroke="#3a2200" strokeWidth="1.2" strokeLinecap="round"/>

          {/* 右耳 */}
          <polygon points="106,66 134,12 156,74"
            fill="#f2d48c" stroke="#3a2200" strokeWidth="2.5" strokeLinejoin="round"/>
          <polygon points="110,64 132,30 147,70" fill="#ffaab8"/>
          <path d="M 131,13 L 127,4 M 134,12 L 134,3 M 137,13 L 142,5"
            stroke="#3a2200" strokeWidth="1.2" strokeLinecap="round"/>

          {/* 胸部绒毛 */}
          <ellipse cx="100" cy="166" rx="30" ry="24" fill="#fffff5"/>

          {/* 额头虎纹（淡） */}
          <path d="M 84,62 Q 92,56 100,62 Q 108,56 116,62"
            fill="none" stroke="#c8943a" strokeWidth="1.8" opacity="0.45"/>

          {/* ── 腮红层（部分情绪显示）── */}
          <g className="pet-blush">
            <g className="expression-happy">
              <ellipse cx="64"  cy="112" rx="11" ry="7" fill="rgba(255,120,120,0.22)"/>
              <ellipse cx="136" cy="112" rx="11" ry="7" fill="rgba(255,120,120,0.22)"/>
            </g>
            <g className="expression-excited">
              <ellipse cx="63"  cy="113" rx="13" ry="8" fill="rgba(255,100,100,0.18)"/>
              <ellipse cx="137" cy="113" rx="13" ry="8" fill="rgba(255,100,100,0.18)"/>
            </g>
          </g>

          {/* ── 眼睛层（Step 2 视线追踪的 transform 目标）── */}
          <g className="pet-eyes">

            {/* idle：标准杏仁眼 */}
            <g className="expression-idle">
              <ellipse cx="82"  cy="100" rx="11" ry="12" fill="#2a1600"/>
              <ellipse cx="85"  cy="97"  rx="3.5" ry="3.5" fill="white"/>
              <ellipse cx="118" cy="100" rx="11" ry="12" fill="#2a1600"/>
              <ellipse cx="121" cy="97"  rx="3.5" ry="3.5" fill="white"/>
            </g>

            {/* happy：弯月眼 ^_^ */}
            <g className="expression-happy">
              <path d="M 71,104 Q 82,91 93,104"
                fill="none" stroke="#2a1600" strokeWidth="2.8" strokeLinecap="round"/>
              <path d="M 107,104 Q 118,91 129,104"
                fill="none" stroke="#2a1600" strokeWidth="2.8" strokeLinecap="round"/>
            </g>

            {/* sleepy：半睁眼 + 沉重眼皮 */}
            <g className="expression-sleepy">
              <ellipse cx="82"  cy="105" rx="11" ry="6"  fill="#2a1600"/>
              <path d="M 71,100 Q 82,96 93,100"
                fill="none" stroke="#2a1600" strokeWidth="2.2" strokeLinecap="round"/>
              <ellipse cx="118" cy="105" rx="11" ry="6"  fill="#2a1600"/>
              <path d="M 107,100 Q 118,96 129,100"
                fill="none" stroke="#2a1600" strokeWidth="2.2" strokeLinecap="round"/>
            </g>

            {/* excited：放大双高光眼 */}
            <g className="expression-excited">
              <circle cx="82"  cy="100" r="14"  fill="#2a1600"/>
              <circle cx="87"  cy="95"  r="4.5" fill="white"/>
              <circle cx="92"  cy="96"  r="2"   fill="white"/>
              <circle cx="118" cy="100" r="14"  fill="#2a1600"/>
              <circle cx="123" cy="95"  r="4.5" fill="white"/>
              <circle cx="128" cy="96"  r="2"   fill="white"/>
            </g>

            {/* curious：一大一小歪头感 */}
            <g className="expression-curious">
              <ellipse cx="82"  cy="100" rx="13"  ry="14"  fill="#2a1600"/>
              <ellipse cx="86"  cy="96"  rx="4"   ry="4"   fill="white"/>
              <ellipse cx="118" cy="100" rx="9"   ry="10"  fill="#2a1600"/>
              <ellipse cx="121" cy="97"  rx="2.5" ry="2.5" fill="white"/>
            </g>

          </g>{/* /pet-eyes */}

          {/* ── 鼻子（静态）── */}
          <path d="M 100,112 L 95,119 L 105,119 Z" fill="#e87890"/>

          {/* ── 嘴巴层 ── */}
          <g className="pet-mouth">

            {/* idle */}
            <g className="expression-idle">
              <path d="M 95,119 Q 100,126 105,119"
                fill="none" stroke="#3a2200" strokeWidth="1.8" strokeLinecap="round"/>
              <path d="M 100,119 L 100,124"
                fill="none" stroke="#3a2200" strokeWidth="1.5" strokeLinecap="round"
                opacity="0.5"/>
            </g>

            {/* happy */}
            <g className="expression-happy">
              <path d="M 89,119 Q 100,133 111,119"
                fill="none" stroke="#3a2200" strokeWidth="2.2" strokeLinecap="round"/>
            </g>

            {/* sleepy */}
            <g className="expression-sleepy">
              <path d="M 96,121 Q 100,124 104,121"
                fill="none" stroke="#3a2200" strokeWidth="1.6" strokeLinecap="round"/>
            </g>

            {/* excited：张嘴 */}
            <g className="expression-excited">
              <ellipse cx="100" cy="126" rx="10" ry="8" fill="#3a2200"/>
            </g>

            {/* curious：轻微不对称 */}
            <g className="expression-curious">
              <path d="M 95,120 Q 102,128 109,119"
                fill="none" stroke="#3a2200" strokeWidth="1.8" strokeLinecap="round"/>
            </g>

          </g>{/* /pet-mouth */}

          {/* ── 胡须（静态）── */}
          <g className="pet-whiskers">
            <line x1="18"  y1="106" x2="84"  y2="110" stroke="#3a2200" strokeWidth="0.9" opacity="0.35" strokeLinecap="round"/>
            <line x1="16"  y1="115" x2="84"  y2="116" stroke="#3a2200" strokeWidth="0.9" opacity="0.35" strokeLinecap="round"/>
            <line x1="20"  y1="124" x2="84"  y2="121" stroke="#3a2200" strokeWidth="0.9" opacity="0.35" strokeLinecap="round"/>
            <line x1="182" y1="106" x2="116" y2="110" stroke="#3a2200" strokeWidth="0.9" opacity="0.35" strokeLinecap="round"/>
            <line x1="184" y1="115" x2="116" y2="116" stroke="#3a2200" strokeWidth="0.9" opacity="0.35" strokeLinecap="round"/>
            <line x1="180" y1="124" x2="116" y2="121" stroke="#3a2200" strokeWidth="0.9" opacity="0.35" strokeLinecap="round"/>
          </g>

        </g>{/* /pet-head */}
      </svg>
    </div>
  );
}
