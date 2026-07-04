window.onload = function () {
  let old_index_for_text_block;
  //waits for sortable.js to load
  var el = document.getElementById("sortable-container");
  if (el) {
    var sortable = Sortable.create(el, {
      handle: ".drag-handle",
      animation: 150,
      onStart: function (evt) {
        // This is the index you want: which child was clicked
        old_index_for_text_block = evt.oldIndex;
        //console.log("Dragged from position:", old_index_for_text_block);
      },
      onEnd: function (evt) {
        var newIndex = evt.newIndex;
        call_rust(old_index_for_text_block, newIndex);
      },
    });
  } else {
    console.error("Element #sortable-container not found");
  }
};

function call_rust(old_id, new_id) {
  const event = new CustomEvent("update_list_order", {
    detail: [String(old_id), String(new_id)],
  });
  window.dispatchEvent(event);
}
