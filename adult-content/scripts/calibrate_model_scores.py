import argparse
import logging

from twitter.adult_content import calibration
from twitter.adult_content.config import CalibrationConfig


def get_argument_parser():
    parser = argparse.ArgumentParser(
        description="Adult model score calibration.",
        argument_default=argparse.SUPPRESS,
    )
    add_dataset_args(parser.add_argument_group("dataset"))
    add_calibration_args(parser.add_argument_group("calibration"))
    return parser


def parse_args():
    parser = get_argument_parser()
    return parser.parse_args()


def add_dataset_args(parser):
    parser.add_argument(
        "--dataset_path",
        help="path to CSV file with predictions and labels; this argument is required",
        type=str,
        required=True,
    )


def add_calibration_args(parser):
    parser.add_argument(
        "--output_dir",
        help="save directory location",
        type=str,
    )
    parser.add_argument(
        "--num_thresholds",
        help="number of thresholds used for calibration",
        type=int,
    )


if __name__ == "__main__":
    config = CalibrationConfig(**vars(parse_args()))

    logging.basicConfig(level=logging.INFO)
    logger = logging.getLogger()
    logger.info(" Adult Content Model Training Configuration:")
    for k, v in config.__dict__.items():
        logger.info(f"\t{k}: {v}")

    calibration.calibrate_and_write_scores(config)
