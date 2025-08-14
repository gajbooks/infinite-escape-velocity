import { inject, Injectable } from "@angular/core";
import { LoginService } from "./login.service";

@Injectable({
    providedIn: 'root',
})
export class SessionService {
    private session = inject(LoginService);

    private current_session_token: string | null = null;

    async getCurrentSessionToken(): Promise<string> {
        if (this.current_session_token != null) {
            return this.current_session_token;
        } else {
            let ephemeral_player = await this.session.createEphemeralPlayer();
            this.current_session_token = await this.session.loginEphemeralPlayer(ephemeral_player);
            return this.current_session_token;
        }
    }
}