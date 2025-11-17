<script setup lang="ts">
import { RouterLink } from "vue-router";

interface Props {
  to: string;
  icon: string;
  label: string;
}

defineProps<Props>();
</script>

<template>
  <RouterLink v-slot="{ navigate, isActive }" :to custom>
    <Button :icon :aria-label="label" :outlined="!isActive" @click="navigate" />
  </RouterLink>
</template>
