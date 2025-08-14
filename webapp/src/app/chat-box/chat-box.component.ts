import { Component, inject } from "@angular/core";
import { ChatService } from "../services/chat.service";

@Component({
providers: [ChatService],
  selector: 'chat-box',
  templateUrl: './chat-box.component.html',
  styleUrls: ['./chat-box.component.less']
})
export class ChatBoxComponent {
    private chat_service = inject(ChatService);

  async ngOnInit() {
    let self = this;
    setInterval(() => {
        self.sendTime();
    }, 5000);

    (await this.chat_service.subscribeToChat()).subscribe((message) => {
        console.log("Received: %s %s", message.player_name, message.message);
    })
  }

  async sendTime() {
    await this.chat_service.sendMessage(new Date(Date.now()).toString());
  }
}