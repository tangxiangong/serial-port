import {
  showSettingPanel,
  setShowSettingPanel,
  showSendPanel,
  setShowSendPanel,
  showQuickInputPanel,
  setShowQuickInputPanel,
  showFullScreen,
  setShowFullScreen,
} from "../stores/ui";

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
          {showSettingPanel() ? (
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24">
              <path fill="currentColor" d="M6 21a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3h12a3 3 0 0 1 3 3v12a3 3 0 0 1-3 3zM18 5h-8v14h8a1 1 0 0 0 1-1V6a1 1 0 0 0-1-1" />
            </svg>
          ) : (
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24">
              <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zm5-2v16" />
            </svg>
          )}
        </button>

        {/* Toggle send panel */}
        <button
          class={btnClass}
          classList={{ "btn-active": showSendPanel() }}
          onClick={() => setShowSendPanel(!showSendPanel())}
          title={showSendPanel() ? "折叠输入面板" : "展开输入面板"}
        >
          {showSendPanel() ? (
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24">
              <path fill="currentColor" d="M18 3a3 3 0 0 1 2.995 2.824L21 6v12a3 3 0 0 1-2.824 2.995L18 21H6a3 3 0 0 1-2.995-2.824L3 18V6a3 3 0 0 1 2.824-2.995L6 3zm0 2H6a1 1 0 0 0-.993.883L5 6v9h14V6a1 1 0 0 0-.883-.993z" />
            </svg>
          ) : (
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24">
              <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zm0 9h16" />
            </svg>
          )}
        </button>

        {/* Toggle quick input panel */}
        <button
          class={btnClass}
          classList={{ "btn-active": showQuickInputPanel() }}
          onClick={() => setShowQuickInputPanel(!showQuickInputPanel())}
          title={showQuickInputPanel() ? "折叠快捷输入面板" : "展开快捷输入面板"}
        >
          {showQuickInputPanel() ? (
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24">
              <path fill="currentColor" d="M6 21a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3h12a3 3 0 0 1 3 3v12a3 3 0 0 1-3 3zm8-16H6a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h8z" />
            </svg>
          ) : (
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24">
              <path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zm11-2v16" />
            </svg>
          )}
        </button>

        {/* Toggle fullscreen */}
        <button
          class={btnClass}
          classList={{ "btn-active": showFullScreen() }}
          onClick={() => setShowFullScreen(!showFullScreen())}
          title={showFullScreen() ? "退出全尺寸" : "全尺寸显示"}
        >
          {showFullScreen() ? (
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24">
              <path fill="currentColor" d="M6 21v-3H3v-2h5v5zm10 0v-5h5v2h-3v3zM3 8V6h3V3h2v5zm13 0V3h2v3h3v2z" />
            </svg>
          ) : (
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24">
              <path fill="currentColor" d="M3 21v-5h2v3h3v2zm13 0v-2h3v-3h2v5zM3 8V3h5v2H5v3zm16 0V5h-3V3h5v5z" />
            </svg>
          )}
        </button>
      </div>
    </div>
  );
}
