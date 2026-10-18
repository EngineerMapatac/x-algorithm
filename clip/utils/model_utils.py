import numpy as np


def create_logits(x1, x2, logit_scale):
    x1 = x1 / x1.norm(dim=-1, keepdim=True)
    x2 = x2 / x2.norm(dim=-1, keepdim=True)

    logits_per_x1 = logit_scale * x1 @ x2.t()
    logits_per_x2 = logits_per_x1.t()

    return logits_per_x1, logits_per_x2


def print_clip_model_stats(model):
    input_resolution = model.visual.input_resolution
    context_length = model.context_length
    vocab_size = model.vocab_size
    print(
        "Model parameters:",
        f"{np.sum([int(np.prod(p.shape)) for p in model.parameters()]):,}",
    )
    print("Input resolution:", input_resolution)
    print("Context length:", context_length)
    print("Vocab size:", vocab_size)


def convert_models_to_fp32(model):
    for p in model.parameters():
        p.data = p.data.float()
        if hasattr(p.grad, "data"):
            p.grad.data = p.grad.data.float()
