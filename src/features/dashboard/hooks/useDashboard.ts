import { useQuery } from "@tanstack/react-query";
import { getDashboard, type DashboardData } from "@/services/api";

export function useDashboard() {
  return useQuery<DashboardData>({
    queryKey: ["dashboard"],
    queryFn: getDashboard,
  });
}
