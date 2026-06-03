import { useQuery } from "@tanstack/react-query";
import { listKanji, getKanjiDetail } from "@/services/api";

export function useKanjiList(page = 1, search?: string) {
  return useQuery({ queryKey: ["kanji", "list", page, search], queryFn: () => listKanji(page, search, 20) });
}

export function useKanjiDetail(id: string) {
  return useQuery({ queryKey: ["kanji", id], queryFn: () => getKanjiDetail(id), enabled: !!id });
}
