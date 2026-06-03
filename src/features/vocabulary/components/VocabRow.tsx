import { useNavigate } from "react-router-dom";
import { Badge } from "@/components/ui/badge";
import type { Vocabulary } from "../types";

interface VocabRowProps { vocab: Vocabulary; }

export default function VocabRow({ vocab }: VocabRowProps) {
  const navigate = useNavigate();
  return (
    <div
      className="flex cursor-pointer items-center justify-between rounded-lg border p-4 transition-colors hover:bg-muted/50"
      onClick={() => navigate(`/vocabulary/${vocab.id}`)}
    >
      <div className="flex items-center gap-4">
        <span className="text-xl font-bold">{vocab.word}</span>
        <span className="text-muted-foreground">{vocab.reading}</span>
      </div>
      <div className="flex items-center gap-3">
        <span className="text-sm">{vocab.meaning}</span>
        {vocab.tags && <Badge variant="outline" className="text-xs">N3</Badge>}
      </div>
    </div>
  );
}
