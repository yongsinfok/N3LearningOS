import { useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { lookupWord } from "@/services/api";
import { Card } from "@/components/ui/card";

interface Props {
  word: string;
  position: { x: number; y: number };
  onClose: () => void;
}

export default function WordPopover({ word, position, onClose }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const { data, isLoading } = useQuery({
    queryKey: ["lookup", word],
    queryFn: () => lookupWord(word),
  });

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    const escHandler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", handler);
    document.addEventListener("keydown", escHandler);
    return () => {
      document.removeEventListener("mousedown", handler);
      document.removeEventListener("keydown", escHandler);
    };
  }, [onClose]);

  return (
    <div
      ref={ref}
      className="fixed z-50"
      style={{ left: position.x, top: position.y + 8 }}
    >
      <Card className="w-72 p-4 shadow-xl">
        {isLoading ? (
          <p className="text-sm text-muted-foreground">查询中...</p>
        ) : data ? (
          <div className="space-y-1">
            <p className="text-lg font-bold">{data.word}</p>
            <p className="text-sm text-muted-foreground">{data.reading}</p>
            <p className="text-sm">{data.meaning}</p>
            {data.example && (
              <p className="mt-2 text-xs italic text-muted-foreground">{data.example}</p>
            )}
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">未在词汇库中找到</p>
        )}
      </Card>
    </div>
  );
}
