import type { Kanji } from "../types";

interface KanjiGridProps {
  kanjiList: Kanji[];
  onSelect: (id: string) => void;
}

export default function KanjiGrid({ kanjiList, onSelect }: KanjiGridProps) {
  return (
    <div className="grid grid-cols-4 gap-3 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10">
      {kanjiList.map((k) => (
        <button
          key={k.id}
          onClick={() => onSelect(k.id)}
          className="flex flex-col items-center gap-1 rounded-lg border p-3 transition-colors hover:bg-muted/50"
        >
          <span className="text-2xl">{k.character}</span>
          <span className="max-w-full truncate text-xs text-muted-foreground">
            {k.onyomi ? JSON.parse(k.onyomi)?.[0] || "" : ""}
          </span>
        </button>
      ))}
    </div>
  );
}
