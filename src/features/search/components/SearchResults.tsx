import { useNavigate } from "react-router-dom";
import { Card } from "@/components/ui/card";
import type { SearchResult } from "@/services/api";

interface SearchResultsProps {
  results: SearchResult[];
  onClose: () => void;
}

const routeMap: Record<string, string> = {
  vocabulary: "/vocabulary/",
  grammar: "/grammar/",
  kanji: "/kanji/",
};

export default function SearchResults({ results, onClose }: SearchResultsProps) {
  const navigate = useNavigate();

  if (results.length === 0) {
    return <p className="p-4 text-sm text-muted-foreground">无结果</p>;
  }

  return (
    <Card className="absolute left-0 right-0 top-full z-50 mt-1 max-h-80 overflow-y-auto">
      {results.map((r) => (
        <button
          key={`${r.content_type}-${r.id}`}
          className="flex w-full items-center justify-between px-4 py-3 text-left transition-colors hover:bg-muted/50"
          onClick={() => {
            navigate(`${routeMap[r.content_type] || "/"}${r.id}`);
            onClose();
          }}
        >
          <div>
            <p className="font-medium">{r.title}</p>
            <p className="text-sm text-muted-foreground">{r.subtitle}</p>
          </div>
          <span className="text-xs text-muted-foreground">{r.content_type}</span>
        </button>
      ))}
    </Card>
  );
}
