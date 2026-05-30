import { useMemo } from "react";
import { createBranchTopology } from "../services/topologyService.js";

export function useBranchTopology({ branches, graphsByBranch, hoveredBranchName, repository, selectedBranchName }) {
  return useMemo(
    () =>
      createBranchTopology({
        branches,
        graphsByBranch,
        hoveredBranchName,
        repository,
        selectedBranchName,
      }),
    [branches, graphsByBranch, hoveredBranchName, repository, selectedBranchName],
  );
}
