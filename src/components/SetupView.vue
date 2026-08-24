<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{
  baseUrl: string;
  configured: boolean;
  busy: boolean;
  error: string;
}>();

const emit = defineEmits<{
  save: [baseUrl: string, token: string];
  cancel: [];
}>();

const url = ref(props.baseUrl);
const token = ref("");

watch(
  () => props.baseUrl,
  (value) => {
    url.value = value;
  },
);

function submit() {
  emit("save", url.value, token.value);
}
</script>

<template>
  <div class="p-3" @dblclick.stop>
    <section class="card bg-base-200 shadow-xs">
      <div class="card-body gap-5">
        <div class="flex items-center gap-3">
          <img src="/app-icon.png" alt="" class="size-12 rounded-box" />
          <div>
            <div class="text-xs font-semibold text-primary">SUBCARD</div>
            <h1 class="card-title text-2xl">连接服务</h1>
          </div>
        </div>

        <p class="text-sm leading-6 opacity-70">
          管理员 API Key 可查看参与者并直接应用建议。系统用户 API Key
          只显示管理员授权的参与者，且不提供“应用建议”；使用前需要开放“参与者”页面权限。
          Key 只保存在系统凭据库中。
        </p>

        <form class="grid gap-4" @submit.prevent="submit">
          <label class="form-control grid gap-2">
            <span class="label-text text-sm">Sub2Pool 地址</span>
            <input
              v-model.trim="url"
              type="url"
              inputmode="url"
              autocomplete="url"
              placeholder="https://pool.example.com"
              class="input w-full"
              required
              :disabled="busy"
            />
          </label>
          <label class="form-control grid gap-2">
            <span class="label-text text-sm">Sub2Pool API Key</span>
            <input
              v-model="token"
              type="password"
              autocomplete="off"
              :placeholder="configured ? '留空则保留当前 Token' : 'sub2pool_…'"
              class="input w-full"
              :required="!configured"
              :disabled="busy"
            />
          </label>

          <div v-if="error" class="alert alert-error text-sm" role="alert">
            {{ error }}
          </div>

          <div class="card-actions justify-end">
            <button
              v-if="configured"
              class="btn"
              type="button"
              :disabled="busy"
              @click="emit('cancel')"
            >
              返回卡片
            </button>
            <button class="btn btn-primary" type="submit" :disabled="busy">
              <span v-if="busy" class="loading loading-spinner loading-sm"></span>
              {{ configured ? "保存并验证" : "连接" }}
            </button>
          </div>
        </form>

        <p class="text-xs opacity-50">
          服务地址会写入本地配置；API Key 不会写入普通配置文件。
        </p>
      </div>
    </section>
  </div>
</template>
