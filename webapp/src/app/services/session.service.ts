import { inject, Injectable } from "@angular/core";
import { LoginService } from "./login.service";

@Injectable({
    providedIn: 'root',
})
export class SessionService {
    private session = inject(LoginService);

    private current_session_token: string | null = null;

    private session_await: null | Promise<string> = null;

    async getCurrentSessionToken(): Promise<string> {
        if (this.current_session_token != null) {
            return this.current_session_token;
        } else {
            if (this.session_await == null) {
                this.session_await = new Promise(async (resolve) => {
                    let ephemeral_player = await this.session.createEphemeralPlayer();

                    resolve(await this.session.loginEphemeralPlayer(ephemeral_player));
                });
            }

            this.current_session_token = await this.session_await;
            return this.current_session_token;
        }
    }
}