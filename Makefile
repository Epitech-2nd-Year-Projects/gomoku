NAME = pbrain-gomoku-ai

all: $(NAME)

$(NAME):
	cargo build --release
	cp target/release/$(NAME) .

clean:
	cargo clean

fclean: clean
	rm -f $(NAME)

re: fclean all

.PHONY: all clean fclean re
