import { ref } from "vue";
import { ElMessageBox } from "element-plus";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export function useAutoUpdate() {
  const showUpdateDialog = ref(false);
  const updateVersion = ref("");
  const updateDownloading = ref(false);
  const updateProgress = ref(0);
  const updateTotal = ref(0);

  async function checkForUpdates() {
    try {
      const update = await check();
      if (update) {
        let userConfirmed = false;
        try {
          await ElMessageBox.confirm(
            `发现新版本 ${update.version}，是否立即更新？`,
            "版本更新",
            {
              confirmButtonText: "立即更新",
              cancelButtonText: "稍后提醒",
              type: "info",
            }
          );
          userConfirmed = true;
        } catch {
          // User clicked cancel
          return;
        }

        if (userConfirmed) {
          // Show download progress dialog
          updateVersion.value = update.version;
          updateProgress.value = 0;
          updateTotal.value = 0;
          updateDownloading.value = true;
          showUpdateDialog.value = true;

          try {
            let downloaded = 0;

            await update.downloadAndInstall((progress) => {
              if (progress.event === "Started" && progress.data) {
                updateTotal.value = progress.data.contentLength || 0;
              } else if (progress.event === "Progress" && progress.data) {
                downloaded += progress.data.chunkLength || 0;
                if (updateTotal.value > 0) {
                  updateProgress.value = Math.round((downloaded / updateTotal.value) * 100);
                }
              } else if (progress.event === "Finished") {
                updateProgress.value = 100;
              }
            });

            updateDownloading.value = false;

            await ElMessageBox.alert("更新已下载完成，点击确定重启应用", "更新完成", {
              confirmButtonText: "重启应用",
            });

            showUpdateDialog.value = false;
            await relaunch();
          } catch (downloadError) {
            console.error("下载更新失败:", downloadError);
            showUpdateDialog.value = false;
            updateDownloading.value = false;

            await ElMessageBox.alert(
              `下载更新失败: ${downloadError instanceof Error ? downloadError.message : "未知错误"}\n\n请稍后重试或手动下载更新。`,
              "更新失败",
              {
                confirmButtonText: "确定",
                type: "error",
              }
            );
          }
        }
      }
    } catch (e) {
      // Silently handle check failure to not affect user experience
      console.log("检查更新失败:", e);
      showUpdateDialog.value = false;
      updateDownloading.value = false;
    }
  }

  return {
    showUpdateDialog,
    updateVersion,
    updateDownloading,
    updateProgress,
    updateTotal,
    checkForUpdates,
  };
}
