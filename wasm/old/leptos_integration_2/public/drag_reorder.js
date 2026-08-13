let oldIndexForTextBlock;

function initSortable() {
    const el = document.getElementById("sortable-container");

    if (!el) {
        console.error("Element #sortable-container not found");
        return;
    }

    Sortable.create(el, {
        handle: ".drag-handle",
        animation: 150,

        onStart(evt) {
            oldIndexForTextBlock = evt.oldIndex;
        },

        onEnd(evt) {
            window.dispatchEvent(new CustomEvent("update_list_order", {
                detail: [
                    String(oldIndexForTextBlock),
                    String(evt.newIndex),
                ],
            }));
        },
    });
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initSortable);
} else {
    initSortable();
}
