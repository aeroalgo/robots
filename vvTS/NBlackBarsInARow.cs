using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000C1 RID: 193
	[HandlerCategory("vvTrade"), HandlerName("N-чёрных баров подряд")]
	public class NBlackBarsInARow : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060006CF RID: 1743 RVA: 0x0001E8AC File Offset: 0x0001CAAC
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
			for (int i = num; i >= barNum - this.NBars; i--)
			{
				if (i == barNum - this.NBars)
				{
					return true;
				}
				if (src.get_ClosePrices()[i] >= src.get_OpenPrices()[i])
				{
					return false;
				}
			}
			return false;
		}

		// Token: 0x17000258 RID: 600
		[HandlerParameter(true, "0", Min = "0", Max = "5", Step = "1")]
		public int backshift
		{
			// Token: 0x060006CD RID: 1741 RVA: 0x0001E89B File Offset: 0x0001CA9B
			get;
			// Token: 0x060006CE RID: 1742 RVA: 0x0001E8A3 File Offset: 0x0001CAA3
			set;
		}

		// Token: 0x17000257 RID: 599
		[HandlerParameter(true, "3", Min = "3", Max = "10", Step = "1")]
		public int NBars
		{
			// Token: 0x060006CB RID: 1739 RVA: 0x0001E88A File Offset: 0x0001CA8A
			get;
			// Token: 0x060006CC RID: 1740 RVA: 0x0001E892 File Offset: 0x0001CA92
			set;
		}
	}
}
