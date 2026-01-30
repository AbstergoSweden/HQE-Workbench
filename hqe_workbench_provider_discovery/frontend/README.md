# Frontend Wiring (Example)

In your React UI, call the `discover_models` Tauri command and populate a dropdown.

Example TypeScript snippet:

```ts
import { invoke } from "@tauri-apps/api/core";

type ProviderModelList = {
  provider_kind: "openai" | "venice" | "openrouter" | "xai" | "generic";
  base_url: string;
  fetched_at_unix_s: number;
  models: { id: string; name: string; context_length?: number }[];
};

export async function refreshModels(base_url: string, api_key: string) {
  return await invoke<ProviderModelList>("discover_models", {
    input: { base_url, api_key, headers: {}, timeout_s: 60, no_cache: false }
  });
}
```
