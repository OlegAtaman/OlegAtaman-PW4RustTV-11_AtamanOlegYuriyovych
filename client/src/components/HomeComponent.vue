<template>
  <div
    class="min-h-screen flex items-center justify-center bg-gradient-to-br from-purple-500 via-pink-500 to-red-500"
  >
    <div
      class="flex flex-col space-y-6 w-full max-w-lg px-10 py-10 bg-white rounded-2xl shadow-2xl border-4 border-purple-600"
    >
      <h1 class="text-4xl font-extrabold text-center text-gray-900">
        Join Us Today!
      </h1>

      <RouterLink :to="routes.SIGNUP">
        <button
          class="btn btn-primary w-full py-3 text-lg transform-none hover:scale-110 hover:shadow-lg transition-transform duration-300"
        >
          Sign Up
        </button>
      </RouterLink>

      <RouterLink :to="routes.LOGIN">
        <button
          class="btn btn-outline btn-secondary w-full py-3 text-lg hover:bg-secondary hover:text-white hover:scale-105 hover:shadow-lg transition-all duration-300"
        >
          Log In
        </button>
      </RouterLink>
    </div>
  </div>
</template>

<script setup>
import { routes } from '../router/routes'
</script>
