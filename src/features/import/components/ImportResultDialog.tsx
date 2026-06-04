import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ImportFileResult } from "@/services/api";

interface Props {
  result: ImportFileResult | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export default function ImportResultDialog({ result, open, onOpenChange }: Props) {
  if (!result) return null;

  const hasErrors = result.errors.length > 0;
  const allFailed = result.imported === 0 && result.total_in_file > 0;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className={allFailed ? "text-destructive" : hasErrors ? "text-amber-500" : "text-green-600"}>
            {allFailed ? "❌ 导入失败" : hasErrors ? "⚠️ 导入完成（有错误）" : "✅ 导入成功"}
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-2 py-4">
          <p>成功导入: <strong>{result.imported}</strong> 条</p>
          {result.overwritten > 0 && (
            <p>覆盖更新: <strong>{result.overwritten}</strong> 条</p>
          )}
          <p className="text-muted-foreground text-sm">文件中共 {result.total_in_file} 条数据</p>
          {hasErrors && (
            <div className="mt-2">
              <p className="text-destructive text-sm font-medium mb-1">
                错误 ({result.errors.length} 条):
              </p>
              <ul className="text-destructive text-xs space-y-1 list-disc pl-4">
                {result.errors.map((err, i) => (
                  <li key={i}>{err}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
        <DialogFooter>
          <Button onClick={() => onOpenChange(false)}>确定</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
