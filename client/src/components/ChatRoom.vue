<template>
  <div
    class="min-h-screen h-screen bg-gradient-to-br from-purple-500 via-pink-500 to-red-500 flex justify-center items-center overflow-hidden"
  >
    <div
      class="bg-gradient-to-br from-purple-500 via-pink-500 to-red-500 rounded-lg shadow-lg w-full p-4 flex flex-col h-full"
    >
      <div class="flex-1 flex flex-col gap-2 overflow-y-auto mt-4 p-8">
        <div
          v-for="(message, index) in messages"
          :key="index"
          :class="
            message.user_id === userId
              ? 'message bg-blue-600 text-white p-3 rounded-lg self-end'
              : 'message bg-gray-700 text-white p-3 rounded-lg self-start'
          "
        >
          <div>
            <span v-if="message.message_type === 'text'">{{
              message.content
            }}</span>

            <div v-else-if="message.message_type === 'file'">
              <a
                :href="`${serverUrl}/api/files/${message.file_path}`"
                download
                :title="message.content"
                class="underline hover:underline"
                target="_blank"
              >
                {{ message.content }}
              </a>
            </div>
          </div>
        </div>
      </div>

      <div class="mt-4 flex items-center gap-3">
        <input
          type="text"
          v-model="newMessage"
          @keyup.enter="sendChatMessage"
          placeholder="Type a message..."
          class="w-full bg-gray-800 text-white rounded-lg p-3 focus:outline-none focus:ring-2 focus:ring-indigo-500"
        />

        <label for="fileInput" class="cursor-pointer">
          <input
            ref="fileInput"
            @change="handleFileChange"
            type="file"
            id="fileInput"
            class="hidden"
          />
          <div
            class="file-input bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg p-3 cursor-pointer transition-all"
          >
            <img
              src="../assets/icons/add-file.png"
              alt="Attach File"
              class="w-6 h-6"
            />
          </div>
        </label>

        <button class="btn btn-primary" @click="sendChatMessage">Send</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useWebSocket } from '@/websockets'
import { useUserStore } from '@/stores/user.store'
import { useRoute } from 'vue-router'

const fileInput = ref(null)
const newMessage = ref('')
const userStore = useUserStore()
const userId = userStore.user.id
const route = useRoute()
const chatId = route.params.chatId
const selectedFile = ref(null)

const serverUrl = import.meta.env.VITE_SERVER_URL

const { messages, sendMessage } = useWebSocket(
  import.meta.env.VITE_WEBSOCKET_URL,
  parseInt(chatId)
)

const sendChatMessage = async () => {
  if (newMessage.value.trim() !== '') {
    if (selectedFile.value) {
      const reader = new FileReader()
      reader.onloadend = () => {
        const dataUrl = reader.result

        const fileUuid = crypto.randomUUID()
        const filePath = `chat_${chatId}_${fileUuid}_${selectedFile.value.name}`

        const message = {
          chat_id: chatId,
          user_id: userId,
          content: newMessage.value,
          file_data: dataUrl,
          file_path: filePath,
          message_type: 'file',
        }

        sendMessage(message)

        newMessage.value = ''
        selectedFile.value = null
        fileInput.value.value = ''
      }
      reader.readAsDataURL(selectedFile.value)
    } else {
      const message = {
        chat_id: chatId,
        user_id: userId,
        content: newMessage.value,
        file_data: null,
        file_path: null,
        message_type: 'text',
      }

      sendMessage(message)
      newMessage.value = ''
    }
  }
}

const handleFileChange = (event) => {
  selectedFile.value = event.target.files[0]
  newMessage.value = selectedFile.value.name
}
</script>
