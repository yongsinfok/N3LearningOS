import { useQuery } from "@tanstack/react-query";
import { listReading, getReadingDetail } from "@/services/api";

export function useReadingList(page = 1, search?: string) {
  return useQuery({
    queryKey: ["reading", "list", page, search],
    queryFn: () => listReading(page, search, 20),
  });
}

export function useReadingDetail(id: string) {
  return useQuery({
    queryKey: ["reading", id],
    queryFn: () => getReadingDetail(id),
    enabled: !!id,
  });
}
