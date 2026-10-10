
      (function () {
        var timeoutMs = 10000;
        // 与 src/lib/update.ts 的 GITHUB_RELEASE_URL 一致（内联脚本不能引用打包产物）。

        window.setTimeout(function () {
          if (window.__WB_MOUNTED__) return;
          var root = document.getElementById("root");
          if (!root) return;

          var wrap = document.createElement("div");
          wrap.className = "wb-boot";

          var title = document.createElement("div");
          title.className = "wb-boot-title";
          title.textContent = "应用启动失败";

          var text = document.createElement("div");
          text.className = "wb-boot-text";
          text.textContent =
            "界面未能正常加载。可以先点「重新加载」再试一次；如果一直失败，请保留错误日志并重新安装本项目安装包。";

          var actions = document.createElement("div");
          actions.className = "wb-boot-actions";

          var reload = document.createElement("button");
          reload.type = "button";
          reload.className = "wb-boot-button";
          reload.textContent = "重新加载";
          reload.addEventListener("click", function () {
            window.location.reload();
          });

          actions.appendChild(reload);

          wrap.appendChild(title);
          wrap.appendChild(text);
          wrap.appendChild(actions);

          root.textContent = "";
          root.appendChild(wrap);
        }, timeoutMs);
      })();
