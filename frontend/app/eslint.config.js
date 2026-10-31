import tseslint from "typescript-eslint";
import kantanda from "../tooling/kantanda-eslint.config.mjs";

export default tseslint.config(
  {
    ignores: ["dist/**", "node_modules/**"],
  },
  ...tseslint.configs.recommended,
  ...kantanda({ maxFnLines: 40, maxDepth: 3, styles: [] }),
);
