import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Upload } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ImportFileResult } from "@/services/api";
import ImportResultDialog from "./ImportResultDialog";
import { ImportContentType } from "../types";
import { useImport } from "../hooks/useImport";

const FILE_FILTERS: Record<ImportContentType, { name: string; extensions: string[] }[]> = {
  vocabulary: [
    { name: "Vocabulary", extensions: ["csv", "json", "apkg"] },
    { name: "CSV", extensions: ["csv"] },
    { name: "JSON", extensions: ["json"] },
    { name: "APKG", extensions: ["apkg"] },
  ],
  grammar: [
    { name: "Grammar", extensions: ["json", "apkg"] },
    { name: "JSON", extensions: ["json"] },
    { name: "APKG", extensions: ["apkg"] },
  ],
  kanji: [
    { name: "Kanji", extensions: ["json", "apkg"] },
    { name: "JSON", extensions: ["json"] },
    { name: "APKG", extensions: ["apkg"] },
  ],
  reading: [{ name: "JSON", extensions: ["json"] }],
  listening: [{ name: "JSON", extensions: ["json"] }],
};

interface Props {
  contentType: ImportContentType;
  onImportComplete: () => void;
}

export default function ImportButton({ contentType, onImportComplete }: Props) {
  const [result, setResult] = useState<ImportFileResult | null>(null);
  const [resultOpen, setResultOpen] = useState(false);
  const [pendingFile, setPendingFile] = useState<string | null>(null);
  const [showModeSelect, setShowModeSelect] = useState(false);
  const importMutation = useImport(contentType);

  const handleClick = async () => {
    try {
      const file = await open({
        multiple: false,
        filters: FILE_FILTERS[contentType],
      });
      if (!file) return;
      setPendingFile(file);
      setShowModeSelect(true);
    } catch (err) {
      console.error("File dialog error:", err);
    }
  };

  const handleModeSelect = async (mode: "Append" | "Overwrite") => {
    if (!pendingFile) return;
    setShowModeSelect(false);
    setPendingFile(null);

    try {
      const res = await importMutation.mutateAsync({ filePath: pendingFile, mode });
      setResult(res);
      setResultOpen(true);
      onImportComplete();
    } catch (err) {
      setResult({
        imported: 0,
        overwritten: 0,
        errors: [typeof err === "string" ? err : "导入失败"],
        total_in_file: 0,
      });
      setResultOpen(true);
    }
  };

  return (
    <>
      <Button
        variant="outline"
        size="sm"
        onClick={handleClick}
        disabled={importMutation.isPending}
      >
        <Upload className="h-4 w-4 mr-1" />
        {importMutation.isPending ? "导入中..." : "导入"}
      </Button>

      {/* Mode selection dialog */}
      <Dialog open={showModeSelect} onOpenChange={setShowModeSelect}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>选择导入模式</DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-4">
            <Button className="w-full justify-start" variant="outline" onClick={() => handleModeSelect("Append")}>
              📥 仅新增 — 跳过重复数据
            </Button>
            <Button className="w-full justify-start" variant="outline" onClick={() => handleModeSelect("Overwrite")}>
              🔄 覆盖已有 — 更新重复数据
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      <ImportResultDialog
        result={result}
        open={resultOpen}
        onOpenChange={setResultOpen}
      />
    </>
  );
}
