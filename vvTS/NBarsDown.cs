using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000BE RID: 190
	[HandlerCategory("vvTrade"), HandlerName("Снижение значения N-баров подряд")]
	public class NBarsDown : IDoubleCompaper1Handler, IOneSourceHandler, IBooleanReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x060006BD RID: 1725 RVA: 0x0001E6B0 File Offset: 0x0001C8B0
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
				else if (price[i] < price[i - 1])
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

		// Token: 0x17000252 RID: 594
		[HandlerParameter(true, "3", Min = "3", Max = "10", Step = "1")]
		public int NBars
		{
			// Token: 0x060006BB RID: 1723 RVA: 0x0001E69C File Offset: 0x0001C89C
			get;
			// Token: 0x060006BC RID: 1724 RVA: 0x0001E6A4 File Offset: 0x0001C8A4
			set;
		}
	}
}
