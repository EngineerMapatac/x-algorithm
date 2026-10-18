import tensorflow as tf
import torch
import transformers


def get_avg(prev_avg, x, n):
    return (prev_avg * n + x) / (n + 1)


def create_optimizer(
    model,
    learning_rate,
    num_training_steps,
    num_warmup_steps=0,
    num_decay_steps=None,
):
    params = list(model.clip_model.parameters())
    if model.top_feedforward:
        if not model.use_gpu:
            image_encoder, text_encoder = model.image_encoder, model.text_encoder
        else:
            image_encoder, text_encoder = (
                model.image_encoder.module,
                model.text_encoder.module,
            )
        params += list(image_encoder.ff.parameters()) + list(
            text_encoder.ff.parameters()
        )
    optimizer = torch.optim.AdamW(
        params=params, lr=learning_rate, betas=(0.9, 0.98), eps=1e-6, weight_decay=0.01
    )

    if num_decay_steps is None:
        num_decay_steps = num_training_steps
    scheduler = transformers.get_cosine_schedule_with_warmup(
        optimizer,
        num_warmup_steps=num_warmup_steps,
        num_training_steps=num_decay_steps,
    )
    return optimizer, scheduler


def set_memory_growth():
    gpus = tf.config.list_physical_devices("GPU")
    if gpus:
        try:
            for gpu in gpus:
                tf.config.experimental.set_memory_growth(gpu, True)
            logical_gpus = tf.config.experimental.list_logical_devices("GPU")
            print(len(gpus), "Physical GPUs,", len(logical_gpus), "Logical GPUs")
        except RuntimeError as error:
            print(error)
