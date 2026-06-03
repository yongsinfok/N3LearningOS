import { useNavigate } from "react-router-dom";
import type { Grammar } from "../types";
interface GrammarRowProps { grammar: Grammar; }
export default function GrammarRow({ grammar }: GrammarRowProps) {
  const navigate = useNavigate();
  return (
    <div className="flex cursor-pointer items-center justify-between rounded-lg border p-4 transition-colors hover:bg-muted/50" onClick={() => navigate(`/grammar/${grammar.id}`)}>
      <div className="flex items-center gap-4">
        <span className="font-mono text-lg font-bold">{grammar.pattern}</span>
      </div>
      <div className="flex items-center gap-3">
        <span className="text-sm text-muted-foreground">{grammar.meaning}</span>
      </div>
    </div>
  );
}
