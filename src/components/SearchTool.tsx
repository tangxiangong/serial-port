import { createSignal } from "solid-js";

export default function SearchTool() {
  const [open, setOpen] = createSignal(false);
  const [query, setQuery] = createSignal("");

  const handleKeyDown = (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "f") {
      e.preventDefault();
      setOpen(true);
    }
    if (e.key === "Escape") {
      setOpen(false);
      setQuery("");
    }
  };

  if (typeof window !== "undefined") {
    window.addEventListener("keydown", handleKeyDown);
  }

  return (
    <>
      <button
        class="btn btn-sm btn-ghost gap-2 absolute left-1/2 -translate-x-1/2 top-1/2 -translate-y-1/2 opacity-60"
        onClick={() => setOpen(true)}
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8" />
          <path d="m21 21-4.3-4.3" />
        </svg>
        搜索记录
      </button>

      {open() && (
        <div class="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]" onClick={() => setOpen(false)}>
          <div class="absolute inset-0 bg-base-100/80 backdrop-blur-sm" />
          <div
            class="relative z-10 w-full max-w-lg rounded-lg border border-base-300 bg-base-100 shadow-lg"
            onClick={(e) => e.stopPropagation()}
          >
            <div class="flex items-center border-b border-base-300 px-3">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-2 opacity-50 shrink-0">
                <circle cx="11" cy="11" r="8" />
                <path d="m21 21-4.3-4.3" />
              </svg>
              <input
                class="input input-ghost flex-1 text-sm focus:outline-none"
                placeholder="搜索记录..."
                value={query()}
                onInput={(e) => setQuery(e.currentTarget.value)}
                autofocus
              />
            </div>
            <div class="max-h-72 overflow-y-auto p-2">
              <div class="py-6 text-center text-sm opacity-50">
                {query().trim() ? "未找到匹配的记录" : "输入关键词搜索收发记录"}
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
