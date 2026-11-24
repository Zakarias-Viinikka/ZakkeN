package z.special_menu

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

enum class OverlayPage { MAIN, ACTIONS }

@Composable
fun CenteredOverlay(
    hasSelection: Boolean,
    onDismiss: () -> Unit,
    onSelectOption: (String) -> Unit,
    onAction: (SelectionAction) -> Unit,
    modifier: Modifier = Modifier
) {
    var page by remember { mutableStateOf(OverlayPage.MAIN) }

    BackHandler(enabled = true) {
        onDismiss()
    }

    Box(
        modifier = modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.5f))
            .clickable(
                interactionSource = remember { MutableInteractionSource() },
                indication = null,
                onClick = onDismiss
            )
    ) {
        Box(
            modifier = Modifier
                .width(320.dp)
                .defaultMinSize(minHeight = 200.dp)
                .align(Alignment.Center)
                .clip(RoundedCornerShape(16.dp))
                .background(Color.White)
                .border(1.dp, Color(0xFFE0E0E0), RoundedCornerShape(16.dp))
                .clickable(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null,
                    onClick = {}
                )
                .padding(16.dp)
        ) {
            // Top-right close button
            Box(
                modifier = Modifier
                    .size(32.dp)
                    .align(Alignment.TopEnd)
                    .clip(CircleShape)
                    .background(Color(0xFFF0F0F0))
                    .clickable { onDismiss() },
                contentAlignment = Alignment.Center
            ) {
                BasicText(
                    text = "✕",
                    style = TextStyle(
                        color = Color.DarkGray,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold
                    )
                )
            }

            // Top-left back button on ACTIONS page
            if (page == OverlayPage.ACTIONS) {
                Box(
                    modifier = Modifier
                        .size(32.dp)
                        .align(Alignment.TopStart)
                        .clip(CircleShape)
                        .background(Color(0xFFF0F0F0))
                        .clickable { page = OverlayPage.MAIN },
                    contentAlignment = Alignment.Center
                ) {
                    BasicText(
                        text = "‹",
                        style = TextStyle(
                            color = Color.DarkGray,
                            fontSize = 20.sp,
                            fontWeight = FontWeight.Bold
                        )
                    )
                }
            }

            if (page == OverlayPage.MAIN) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 16.dp, bottom = 8.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(20.dp)
                ) {
                    BasicText(
                        text = "Place Marker",
                        style = TextStyle(
                            fontSize = 16.sp,
                            fontWeight = FontWeight.Bold,
                            color = Color(0xFF2D3748)
                        )
                    )

                    Row(
                        horizontalArrangement = Arrangement.spacedBy(24.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Box(
                            modifier = Modifier
                                .size(70.dp)
                                .clip(RoundedCornerShape(12.dp))
                                .background(Color(0xFFE3F2FD))
                                .border(1.5.dp, Color(0xFF2196F3), RoundedCornerShape(12.dp))
                                .clickable { onSelectOption("1") },
                            contentAlignment = Alignment.Center
                        ) {
                            BasicText(
                                text = "1",
                                style = TextStyle(
                                    fontSize = 28.sp,
                                    fontWeight = FontWeight.Bold,
                                    color = Color(0xFF1976D2)
                                )
                            )
                        }

                        Box(
                            modifier = Modifier
                                .size(70.dp)
                                .clip(RoundedCornerShape(12.dp))
                                .background(Color(0xFFE8F5E9))
                                .border(1.5.dp, Color(0xFF4CAF50), RoundedCornerShape(12.dp))
                                .clickable { onSelectOption("2") },
                            contentAlignment = Alignment.Center
                        ) {
                            BasicText(
                                text = "2",
                                style = TextStyle(
                                    fontSize = 28.sp,
                                    fontWeight = FontWeight.Bold,
                                    color = Color(0xFF388E3C)
                                )
                            )
                        }
                    }

                    if (hasSelection) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(0.85f)
                                .clip(RoundedCornerShape(8.dp))
                                .background(Color(0xFFF1F5F9))
                                .border(1.dp, Color(0xFFCBD5E1), RoundedCornerShape(8.dp))
                                .clickable { page = OverlayPage.ACTIONS }
                                .padding(vertical = 10.dp),
                            contentAlignment = Alignment.Center
                        ) {
                            BasicText(
                                text = "Selection Actions →",
                                style = TextStyle(
                                    fontSize = 15.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    color = Color(0xFF1E293B)
                                )
                            )
                        }
                    }
                }
            } else {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 20.dp, bottom = 8.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    BasicText(
                        text = "Selection Actions",
                        style = TextStyle(
                            fontSize = 16.sp,
                            fontWeight = FontWeight.Bold,
                            color = Color(0xFF2D3748)
                        )
                    )

                    Column(
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                        modifier = Modifier.fillMaxWidth(0.9f)
                    ) {
                        Row(
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            ActionButton(
                                label = "Copy",
                                color = Color(0xFF1E88E5),
                                modifier = Modifier.weight(1f)
                            ) {
                                onAction(SelectionAction.COPY)
                            }
                            ActionButton(
                                label = "Cut out",
                                color = Color(0xFFFB8C00),
                                modifier = Modifier.weight(1f)
                            ) {
                                onAction(SelectionAction.CUT)
                            }
                        }
                        Row(
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            ActionButton(
                                label = "Delete",
                                color = Color(0xFFE53935),
                                modifier = Modifier.weight(1f)
                            ) {
                                onAction(SelectionAction.DELETE)
                            }
                            ActionButton(
                                label = "Clear",
                                color = Color(0xFF757575),
                                modifier = Modifier.weight(1f)
                            ) {
                                onAction(SelectionAction.CLEAR)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ActionButton(
    label: String,
    color: Color,
    modifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    Box(
        modifier = modifier
            .clip(RoundedCornerShape(8.dp))
            .background(color.copy(alpha = 0.1f))
            .border(1.dp, color, RoundedCornerShape(8.dp))
            .clickable(onClick = onClick)
            .padding(vertical = 12.dp),
        contentAlignment = Alignment.Center
    ) {
        BasicText(
            text = label,
            style = TextStyle(
                fontSize = 14.sp,
                fontWeight = FontWeight.Bold,
                color = color
            )
        )
    }
}