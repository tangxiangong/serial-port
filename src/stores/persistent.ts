import { Store } from "@tauri-apps/plugin-store";
import { createSignal } from "solid-js";

let store: Store | null = null;

export async function initStore() {
  store = await Store.load("settings.json");
}

export async function getSetting<T>(key: string, defaultValue: T): Promise<T> {
  if (!store) return defaultValue;
  const val = await store.get<T>(key);
  return val ?? defaultValue;
}

export async function setSetting<T>(key: string, value: T): Promise<void> {
  if (!store) return;
  await store.set(key, value);
  await store.save();
}

// Reactive settings helpers
export function createPersistentSignal<T>(key: string, defaultValue: T) {
  const [value, setValue] = createSignal<T>(defaultValue);

  // Load from store on mount
  const load = async () => {
    const stored = await getSetting<T>(key, defaultValue);
    setValue(() => stored as any);
  };

  // Save when value changes
  const setAndPersist = (newValue: T | ((prev: T) => T)) => {
    const resolved = typeof newValue === "function"
      ? (newValue as (prev: T) => T)(value())
      : newValue;
    setValue(() => resolved as any);
    setSetting(key, resolved);
  };

  return [value, setAndPersist, load] as const;
}
