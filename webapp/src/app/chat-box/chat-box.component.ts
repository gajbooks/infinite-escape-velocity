import { Component, inject } from "@angular/core";
import { ChatService } from "../services/chat.service";
import { FormControl, FormGroup, ReactiveFormsModule } from "@angular/forms";

const MAX_MESSAGE_COUNT: number = 20;

@Component({
providers: [ChatService],
  imports: [ReactiveFormsModule],
  selector: 'chat-box',
  templateUrl: './chat-box.component.html',
  styleUrls: ['./chat-box.component.less']
})
export class ChatBoxComponent {
    private chatService = inject(ChatService);

    chatInputBox = new FormControl('');

    chatSubmitGroup = new FormGroup([this.chatInputBox]);

    messages: {name: string, message: string}[] = [];

  async ngOnInit() {
    let self = this;

    (await this.chatService.subscribeToChat()).subscribe((message) => {
        self.receive(message.player_name, message.message);
        console.log("Received: %s %s", message.player_name, message.message);
    })
  }

  async send() {
    let message = this.chatInputBox.value;
    if (message != null) {
        await this.chatService.sendMessage(message);
    }
    this.chatInputBox.setValue(null);
  }

  receive(name: string | null, message: string) {
    this.messages.push({name: name ?? "???", message});
    if (this.messages.length > MAX_MESSAGE_COUNT) {
        this.messages.shift();
    }
  }
}