using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000BD RID: 189
	[HandlerCategory("vvTrade"), HandlerName("Рост значения N-баров подряд")]
	public class NBarsGrouth : IDoubleCompaper1Handler, IOneSourceHandler, IBooleanReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x060006B9 RID: 1721 RVA: 0x0001E61C File Offset: 0x0001C81C
		public IList<bool> Execute(IList<double> price)
		{
			if (this.NBars <= 0)
			{
				this.NBars = 1;
			}
			bool[] array = new bool[price.Count];
			int num = 0;
			for (int i = 0; i < price.Count; i++)
			{
				array[i] = false;
				if (i < this.NBars)
				{
					num = 0;
				}
				else if (price[i] > price[i - 1])
				{
					num++;
				}
				else
				{
					num = 0;
				}
				array[i] = ((num >= this.NBars) ? true : false);
			}
			return array;
		}

		// Token: 0x17000251 RID: 593
		[HandlerParameter(true, "3", Min = "3", Max = "10", Step = "1")]
		public int NBars
		{
			// Token: 0x060006B7 RID: 1719 RVA: 0x0001E60A File Offset: 0x0001C80A
			get;
			// Token: 0x060006B8 RID: 1720 RVA: 0x0001E612 File Offset: 0x0001C812
			set;
		}
	}
}
