using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000C0 RID: 192
	[HandlerCategory("vvTrade"), HandlerName("N-белых баров подряд")]
	public class NWhiteBarsInARow : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006C9 RID: 1737 RVA: 0x0001E804 File Offset: 0x0001CA04
		public bool Execute(ISecurity src, int barNum)
		{
			if (this.NBars <= 0)
			{
				this.NBars = 1;
			}
			if (this.NBars > barNum)
			{
				return false;
			}
			if (this.backshift < 0)
			{
				this.backshift = 0;
			}
			int num = barNum - this.backshift;
			for (int i = num; i >= num - this.NBars; i--)
			{
				if (i == num - this.NBars)
				{
					return true;
				}
				if (src.get_ClosePrices()[i] <= src.get_OpenPrices()[i])
				{
					return false;
				}
			}
			return false;
		}

		// Token: 0x17000256 RID: 598
		[HandlerParameter(true, "0", Min = "0", Max = "5", Step = "1")]
		public int backshift
		{
			// Token: 0x060006C7 RID: 1735 RVA: 0x0001E7F3 File Offset: 0x0001C9F3
			get;
			// Token: 0x060006C8 RID: 1736 RVA: 0x0001E7FB File Offset: 0x0001C9FB
			set;
		}

		// Token: 0x17000255 RID: 597
		[HandlerParameter(true, "3", Min = "3", Max = "10", Step = "1")]
		public int NBars
		{
			// Token: 0x060006C5 RID: 1733 RVA: 0x0001E7E2 File Offset: 0x0001C9E2
			get;
			// Token: 0x060006C6 RID: 1734 RVA: 0x0001E7EA File Offset: 0x0001C9EA
			set;
		}
	}
}
