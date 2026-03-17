import { useEffect, useRef } from "react";

export type PetBehavior = "blink" | "stretch" | "yawn";

const BEHAVIORS: PetBehavior[] = ["blink", "stretch", "yawn"];

/** 触发间隔范围：30 ~ 120 秒 */
const MIN_MS = 30_000;
const MAX_MS = 120_000;

function nextInterval() {
  return MIN_MS + Math.random() * (MAX_MS - MIN_MS);
}

/**
 * 随机行为触发器。
 *
 * 每隔 30-120 秒随机选择一个行为（眨眼 / 伸懒腰 / 打哈欠）并通过回调通知。
 * 回调引用变化不会重置定时器（内部用 ref 保持最新引用）。
 */
export function useRandomBehavior(onBehavior: (b: PetBehavior) => void): void {
  const callbackRef = useRef(onBehavior);
  callbackRef.current = onBehavior;

  useEffect(() => {
    let timerId: ReturnType<typeof setTimeout>;

    const schedule = () => {
      timerId = setTimeout(() => {
        const behavior = BEHAVIORS[Math.floor(Math.random() * BEHAVIORS.length)];
        callbackRef.current(behavior);
        schedule();
      }, nextInterval());
    };

    schedule();

    return () => clearTimeout(timerId);
  }, []);
}
