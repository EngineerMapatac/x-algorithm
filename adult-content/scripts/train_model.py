import argparse
import logging

from twitter.adult_content import training
from twitter.adult_content.config import TrainingConfig


def get_argument_parser():
    parser = argparse.ArgumentParser(
        description="Adult model training.",
        argument_default=argparse.SUPPRESS,
    )
    add_dataset_args(parser.add_argument_group("dataset"))
    add_training_args(parser.add_argument_group("training"))
    return parser


def parse_args():
    parser = get_argument_parser()
    return parser.parse_args()


def add_dataset_args(parser):
    parser.add_argument(
        "--train_dataset_path",
        help="path to GCS dataset with (embedding, labels) pair TFRecords; "
        "this argument is required",
        type=str,
        required=True,
    )
    parser.add_argument(
        "--eval_dataset_path",
        help="path to GCS dataset with (embedding, labels) pair TFRecords; "
        "this argument is required",
        type=str,
        required=True,
    )
    parser.add_argument(
        "--resample",
        help="resample the training dataset to 50/50 distribution",
        action="store_true",
    )
    parser.add_argument(
        "--repeat",
        help="repeat the samples in the training dataset indefinitely",
        action="store_true",
    )


def add_training_args(parser):
    parser.add_argument(
        "--model_dir",
        help="save directory location",
        type=str,
    )
    parser.add_argument(
        "--input_embedding_dim",
        help="embedding dimension",
        type=int,
    )
    parser.add_argument(
        "--dense_layer_units",
        help="number of units in the first dense layer",
        type=int,
    )
    parser.add_argument(
        "--batch_size",
        help="number of samples per batch",
        type=int,
    )
    parser.add_argument(
        "--learning_rate",
        help="learning rate for optimizer",
        type=float,
    )
    parser.add_argument(
        "--num_epochs",
        help="total number of training epochs",
        type=int,
    )
    parser.add_argument(
        "--num_steps_per_epoch",
        help="number of steps per epoch",
        type=int,
    )


if __name__ == "__main__":
    config = TrainingConfig(**vars(parse_args()))

    logging.basicConfig(level=logging.INFO)
    logger = logging.getLogger()
    logger.info(" Adult Content Model Training Configuration:")
    for k, v in config.__dict__.items():
        logger.info(f"\t{k}: {v}")

    training.train_model(config)
