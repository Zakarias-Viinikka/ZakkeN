import argparse

from two_browser_stage import TwoBrowserStage


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", default="about:blank")
    args = parser.parse_args()

    stage = TwoBrowserStage.launch(args.url)
    print("1=brave full  2=side by side  3=edge full  q=quit")

    try:
        while True:
            cmd = input("> ").strip()
            if cmd == "q":
                break
            if cmd == "1":
                stage.make_brave_big()
            elif cmd == "2":
                stage.share_screen_with_edge_and_brave()
            elif cmd == "3":
                stage.make_edge_big()
            else:
                print("unknown: 1 / 2 / 3 / q")
    finally:
        stage.close()


if __name__ == "__main__":
    main()