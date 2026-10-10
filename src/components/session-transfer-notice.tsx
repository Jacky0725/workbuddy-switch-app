/** References in a conversation do not transfer the referenced files. */
export function SessionTransferNotice() {
  return (
    <p role="note" className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-900 dark:border-amber-900 dark:bg-amber-950/30 dark:text-amber-200">
      复制或同步会话不会自动复制附件和本地文件。目标账号可能无法访问原引用，请单独备份并检查这些文件。
    </p>
  );
}
