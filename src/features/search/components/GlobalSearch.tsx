import { useState, useRef, useEffect } from "react";
import { Input } from "@/components/ui/input";
import { useGlobalSearch } from "../hooks/useGlobalSearch";
import SearchResults from "./SearchResults";

export default function GlobalSearch() {
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const { data: results } = useGlobalSearch(query);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  useEffect(() => {
    setOpen(query.length >= 2);
  }, [query]);

  return (
    <div ref={ref} className="relative flex-1 max-w-md">
      <Input
        placeholder="搜索词汇、文法、汉字..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onFocus={() => {
          if (query.length >= 2) setOpen(true);
        }}
      />
      {open && results && <SearchResults results={results} onClose={() => { setOpen(false); setQuery(""); }} />}
    </div>
  );
}
