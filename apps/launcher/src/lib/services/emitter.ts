// Минимальный типизированный эмиттер событий для сервисов.

type Handlers<E> = { [K in keyof E]?: Set<E[K]> };

export class Emitter<E extends { [K in keyof E]: (...args: any[]) => void }> {
  private handlers: Handlers<E> = {};

  on<K extends keyof E>(event: K, handler: E[K]): () => void {
    const set = (this.handlers[event] ??= new Set<E[K]>());
    set.add(handler);
    return () => set.delete(handler);
  }

  emit<K extends keyof E>(event: K, ...args: Parameters<E[K]>) {
    this.handlers[event]?.forEach((h) => h(...args));
  }
}
