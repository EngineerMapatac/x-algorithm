from dataclasses import dataclass, field


@dataclass
class MultimodalPostEmbeddingState:
    embeddings: dict[str, list[float]] = field(default_factory=dict)
