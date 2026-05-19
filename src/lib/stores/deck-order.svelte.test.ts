import { describe, expect, it, beforeEach } from "vitest";
import { deckOrder } from "./deck-order.svelte";

describe("deckOrder.applyOrder", () => {
  beforeEach(() => {
    deckOrder.set([]);
  });

  it("returns input order when nothing is stored", () => {
    expect(deckOrder.applyOrder([10, 20, 30])).toEqual([10, 20, 30]);
  });

  it("orders by stored ids for known items", () => {
    deckOrder.set([30, 10, 20]);
    expect(deckOrder.applyOrder([10, 20, 30])).toEqual([30, 10, 20]);
  });

  it("appends unknown ids at the end preserving input order", () => {
    deckOrder.set([20]);
    expect(deckOrder.applyOrder([10, 20, 30, 40])).toEqual([20, 10, 30, 40]);
  });

  it("drops stored ids that are no longer present (deleted decks)", () => {
    deckOrder.set([999, 10, 30]);
    expect(deckOrder.applyOrder([10, 20, 30])).toEqual([10, 30, 20]);
  });
});

describe("deckOrder.move", () => {
  beforeEach(() => {
    deckOrder.set([]);
  });

  it("moves source immediately before the given target", () => {
    deckOrder.move([10, 20, 30, 40], 30, 10);
    expect(deckOrder.ids).toEqual([30, 10, 20, 40]);
  });

  it("moves source to the end when target is null", () => {
    deckOrder.move([10, 20, 30], 10, null);
    expect(deckOrder.ids).toEqual([20, 30, 10]);
  });

  it("is idempotent when dropping on the source itself (no rearrangement needed)", () => {
    deckOrder.set([10, 20, 30]);
    // 同じ位置に戻す: deckOrder.move(topIds, 20, 30) は 20 を 30 の前に再配置するが
    // 既に 20 は 30 の前にあるので順序は変わらない。
    deckOrder.move([10, 20, 30], 20, 30);
    expect(deckOrder.ids).toEqual([10, 20, 30]);
  });

  it("preserves the position of items that are not the source", () => {
    deckOrder.set([10, 20, 30, 40, 50]);
    deckOrder.move([10, 20, 30, 40, 50], 50, 20);
    expect(deckOrder.ids).toEqual([10, 50, 20, 30, 40]);
  });

  it("uses the current input order when stored order is empty (first-time move)", () => {
    // No prior order. Should treat input as the starting order.
    deckOrder.move([10, 20, 30], 30, 10);
    expect(deckOrder.ids).toEqual([30, 10, 20]);
  });
});
