import { openUrl } from "@tauri-apps/plugin-opener";
import { theme, toggleTheme } from "../stores/ui";
import {
  GithubIcon,
  MoonIcon,
  SunIcon,
  MenuIcon,
  SettingsIcon,
} from "../assets/icons";

export default function ActivityBar() {
  const btnClass = "btn btn-ghost btn-xs btn-square";

  const handleLink = (url: string) => (e: MouseEvent) => {
    e.preventDefault();
    openUrl(url);
  };

  return (
    <div class="flex flex-col h-full p-2 justify-between">
      <div class="flex flex-col items-center gap-2">
        {/* GitHub */}
        <a
          href="https://github.com/tangxiangong/serial-port"
          target="_blank"
          class={btnClass}
          onClick={handleLink("https://github.com/tangxiangong/serial-port")}
        >
          <GithubIcon class="w-4 h-4" />
        </a>
      </div>

      <div class="flex flex-col items-center gap-2">
        {/* Theme toggle */}
        <button class={btnClass} onClick={toggleTheme} title="切换主题">
          {theme() === "dark" ? (
            <MoonIcon class="w-4 h-4" />
          ) : (
            <SunIcon class="w-4 h-4" />
          )}
        </button>

        {/* Menu */}
        <button class={btnClass} title="菜单">
          <MenuIcon class="w-4 h-4" />
        </button>

        {/* Settings */}
        <button class={btnClass} title="设置">
          <SettingsIcon class="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
