# lib/util.py
# util.py stores the general function for checking validity of an dictionary object

def get_board_info(in_dict, board_map):
    """Check if input dictionary is a board and return board info dictionary.

    Arguments:
    in_dict - Input dictionary
    board_map - Mapping between board model and board info
    """
    if check_board(in_dict):
        BOARD_MODEL = in_dict['env']['board']

        if check_board_mapping(BOARD_MODEL, board_map):
            return board_map['boards'][BOARD_MODEL]

    return None

def check_board(in_dict):
    """ Check if input dictionary is in board foramt

    Arguments:
    in_dict - Input dictionary
    """
    return 'env' in in_dict and 'board' in in_dict['env']

def check_board_mapping(board_model, board_map):
    """ Check if the board model exists in the board mapping

    Arguments:
    board_model - Board model name
    board_map - Mapping between board model and board info
    """
    return 'boards' in board_map and board_model in board_map['boards']
