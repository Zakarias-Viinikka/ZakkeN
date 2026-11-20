// Restore the saved values when the page loads
document.getElementById('edit-row-id').value = localStorage.getItem('edit-row-id') || '';
document.getElementById('edit-col').value   = localStorage.getItem('edit-col')   || '';
document.getElementById('edit-val').value = localStorage.getItem('edit-val') || '';
//delete
document.getElementById('delete-row-id').value = localStorage.getItem('delete-row-id') || '';
// Restore swap values
document.getElementById('swap-row-id-1').value = localStorage.getItem('swap-row-id-1') || '';
document.getElementById('swap-row-id-2').value = localStorage.getItem('swap-row-id-2') || '';
document.getElementById('swap-col').value = localStorage.getItem('swap-col') || '';
// restore create table stuff
 document.getElementById('new-table-name').value = localStorage.getItem('new-table-name') || '';
