import { group } from '@angular/animations';
import { Component, ElementRef, Input, ViewChild } from '@angular/core';
import { ClientServerMessage } from 'bindings/ClientServerMessage';
import { DynamicObjectCreationData } from 'bindings/DynamicObjectCreationData';
import { DynamicObjectDestructionData } from 'bindings/DynamicObjectDestructionData';
import { DynamicObjectMessageData } from 'bindings/DynamicObjectMessageData';
import { ServerClientMessage } from 'bindings/ServerClientMessage';
import Konva from 'konva';
import { Observable, Subject, interval } from 'rxjs';

class RenderShip {
  x: number;
  y: number;
  graphics: Konva.Image

  constructor(x: number, y: number, graphics: Konva.Image) {
    this.x = x;
    this.y = y;
    this.graphics = graphics;
  }
}

@Component({
  selector: 'app-gameplay-canvas',
  standalone: true,
  imports: [],
  templateUrl: './gameplay-canvas.component.html',
  styleUrl: './gameplay-canvas.component.less'
})
export class GameplayCanvasComponent {
  @Input({ required: true }) incomingMessages!: Subject<ServerClientMessage>;
  @Input({ required: true }) outgoingMessages!: Subject<ClientServerMessage>;
  @ViewChild('gameWindow') gameWindow!: ElementRef;

  renderer!: Konva.Stage;
  shipLayer!: Konva.Layer;
  renderLoop = interval(16);
  shipImage = new Image();
  ships: Map<BigInt, RenderShip> = new Map();

  constructor() {
    this.shipImage.src = 'http://localhost:2718/data/images/default.webp';
  }

  refreshScreen() {
    let x_offset = this.renderer.width() / 2;
    let y_offset = this.renderer.height() / 2;
    this.ships.forEach((val) => {
      val.graphics.x(val.x + x_offset);
      val.graphics.y(val.y + y_offset);
    });

    this.renderer.draw();
  }

  resizeRenderer() {
    if (this.renderer != null && this.gameWindow != null) {
      if(this.renderer.width() != this.gameWindow.nativeElement.clientWidth) {
        this.renderer.width(this.gameWindow.nativeElement.clientWidth);
      }
      if(this.renderer.height() != this.gameWindow.nativeElement.clientHeight) {
        this.renderer.height(this.gameWindow.nativeElement.clientHeight);
      }
    }
  }

  ngOnInit() {
    this.renderer = new Konva.Stage({
      container: 'gameWindow',
      width: 1000,
      height: 1000
    });
    this.shipLayer = new Konva.Layer();
    this.renderer.add(this.shipLayer);
    this.renderLoop.subscribe(() => {
      this.refreshScreen();
    });
    let canvas = this;
    this.incomingMessages.subscribe({
      next(val) {
        if (val.type == 'DynamicObjectCreation') {
          let new_ship = <DynamicObjectCreationData>val;
          // We don't care about imaginary ships right now
        }

        if (val.type == 'DynamicObjectUpdate') {
          let updated_ship = <DynamicObjectMessageData>val;
          if (canvas.ships.has(updated_ship.id) == false) {
            canvas.ships.set(updated_ship.id, new RenderShip(
              updated_ship.x,
              updated_ship.y,
              new Konva.Image({
                image: canvas.shipImage,
                offsetX: canvas.shipImage.width / 2,
                offsetY: canvas.shipImage.height / 2
              })
            ));
            let ship = <RenderShip>canvas.ships.get(updated_ship.id);
            canvas.shipLayer.add(ship.graphics);
          }

          let ship = canvas.ships.get(updated_ship.id);

          if(typeof ship !== 'undefined') {
            ship.x = updated_ship.x;
            ship.y = updated_ship.y;
          }

        }

        if (val.type == 'DynamicObjectDestruction') {
          let deleted_ship = <DynamicObjectDestructionData>val;
          let ship = canvas.ships.get(deleted_ship.id);

          if(typeof ship !== 'undefined') {
            ship.graphics.remove();
            canvas.ships.delete(deleted_ship.id);
          }
        }

      }
    })
  }

  ngAfterViewChecked() {
    this.resizeRenderer();
  }
}
