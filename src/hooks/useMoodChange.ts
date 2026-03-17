import { useEffect, useRef } from "react";
import type { PetMood } from "../types/pet";

/** 参与自然轮换的情绪（Excited / Curious 仅由交互触发） */
const NATURAL_MOODS: PetMood[] = ["idle", "happy", "sleepy"];

/** 情绪转移表：每种自然情绪可切换到哪些状态 */
const TRANSITIONS: Record<string, PetMood[]> = {
  idle:   ["happy", "sleepy"],
  happy:  ["idle",  "sleepy"],
  sleepy: ["idle",  "happy"],
};

/** 变化间隔范围：5 ~ 10 分钟 */
const MIN_MS = 5 * 60_000;
const MAX_MS = 10 * 60_000;

function nextInterval() {
  return MIN_MS + Math.random() * (MAX_MS - MIN_MS);
}

/**
 * 自然情绪变化系统。
 *
 * 每隔 5-10 分钟在 idle / happy / sleepy 之间随机切换一次。
 * 若当前情绪是 excited 或 curious（交互态），跳过本次切换，定时器继续运行。
 */
export function useMoodChange(
  currentMood: PetMood,
  onMoodChange: (mood: PetMood) => void,
): void {
  const moodRef = useRef(currentMood);
  moodRef.current = currentMood;

  const callbackRef = useRef(onMoodChange);
  callbackRef.current = onMoodChange;

  useEffect(() => {
    let timerId: ReturnType<typeof setTimeout>;

    const schedule = () => {
      timerId = setTimeout(() => {
        const mood = moodRef.current;

        if (NATURAL_MOODS.includes(mood)) {
          const options = TRANSITIONS[mood] ?? ["idle"];
          const next = options[Math.floor(Math.random() * options.length)];
          callbackRef.current(next);
        }
        // 无论是否切换，都重新调度下一次

        schedule();
      }, nextInterval());
    };

    schedule();

    return () => clearTimeout(timerId);
  }, []);
}
