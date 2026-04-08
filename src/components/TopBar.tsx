import {
  setShowFullScreen,
  setShowQuickInputPanel,
  setShowSendPanel,
  setShowSettingPanel,
  showFullScreen,
  showQuickInputPanel,
  showSendPanel,
  showSettingPanel,
} from "../stores/ui.ts";
import {
  FullscreenExitIcon,
  FullscreenIcon,
  PanelBottomActiveIcon,
  PanelBottomIcon,
  PanelLeftActiveIcon,
  PanelLeftIcon,
  PanelRightActiveIcon,
  PanelRightIcon,
} from "../assets/icons";

export default function TopBar() {
  const btnClass = "btn btn-ghost btn-xs btn-square";

  return (
    <div class="p-2 flex items-center justify-end relative">
      <div class="flex items-center gap-1">
        {/* Toggle setting panel */}
        <button
          class={btnClass}
          classList={{ "btn-active": showSettingPanel() }}
          onClick={() => setShowSettingPanel(!showSettingPanel())}
          title={showSettingPanel() ? "折叠设备设置面板" : "展开设备设置面板"}
        >
          {showSettingPanel()
            ? <PanelLeftActiveIcon class="w-[18px] h-[18px]" />
            : <PanelLeftIcon class="w-[18px] h-[18px]" />}
        </button>

        {/* Toggle send panel */}
        <button
          class={btnClass}
          classList={{ "btn-active": showSendPanel() }}
          onClick={() => setShowSendPanel(!showSendPanel())}
          title={showSendPanel() ? "折叠输入面板" : "展开输入面板"}
        >
          {showSendPanel()
            ? <PanelBottomActiveIcon class="w-[18px] h-[18px]" />
            : <PanelBottomIcon class="w-[18px] h-[18px]" />}
        </button>

        {/* Toggle quick input panel */}
        <button
          class={btnClass}
          classList={{ "btn-active": showQuickInputPanel() }}
          onClick={() => setShowQuickInputPanel(!showQuickInputPanel())}
          title={showQuickInputPanel()
            ? "折叠快捷输入面板"
            : "展开快捷输入面板"}
        >
          {showQuickInputPanel()
            ? <PanelRightActiveIcon class="w-[18px] h-[18px]" />
            : <PanelRightIcon class="w-[18px] h-[18px]" />}
        </button>

        {/* Toggle fullscreen */}
        <button
          class={btnClass}
          classList={{ "btn-active": showFullScreen() }}
          onClick={() => setShowFullScreen(!showFullScreen())}
          title={showFullScreen() ? "退出全尺寸" : "全尺寸显示"}
        >
          {showFullScreen()
            ? <FullscreenExitIcon class="w-4 h-4" />
            : <FullscreenIcon class="w-4 h-4" />}
        </button>
      </div>
    </div>
  );
}
