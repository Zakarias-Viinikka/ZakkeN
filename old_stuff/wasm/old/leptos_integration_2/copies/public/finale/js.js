(function() {
    let old_index_for_text_block;
    var el = document.getElementById("sortable-container");

    if (!el) {
        console.error("Element #sortable-container not found");
        return;
    }

    var sortable = Sortable.create(el, {
        handle: ".drag-handle",
        animation: 150,
        onStart: function (evt) {
            old_index_for_text_block = evt.oldIndex;
        },
        onEnd: function (evt) {
            var newIndex = evt.newIndex;
            const event = new CustomEvent("update_list_order", {
                detail: [String(old_index_for_text_block), String(newIndex)],
            });
            window.dispatchEvent(event);
        },
    });
})();
