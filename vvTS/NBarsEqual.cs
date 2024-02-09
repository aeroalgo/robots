using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000BF RID: 191
	[HandlerCategory("vvTrade"), HandlerName("Одинаковое значение N-баров подряд")]
	public class NBarsEqual : IDoubleCompaper1Handler, IOneSourceHandler, IBooleanReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x060006C3 RID: 1731 RVA: 0x0001E754 File Offset: 0x0001C954
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
				else if (price[i - this.Shift] == price[i - 1 - this.Shift])
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

		// Token: 0x17000253 RID: 595
		[HandlerParameter(true, "3", Min = "3", Max = "10", Step = "1")]
		public int NBars
		{
			// Token: 0x060006BF RID: 1727 RVA: 0x0001E730 File Offset: 0x0001C930
			get;
			// Token: 0x060006C0 RID: 1728 RVA: 0x0001E738 File Offset: 0x0001C938
			set;
		}

		// Token: 0x17000254 RID: 596
		[HandlerParameter(true, "0", Min = "0", Max = "3", Step = "1")]
		public int Shift
		{
			// Token: 0x060006C1 RID: 1729 RVA: 0x0001E741 File Offset: 0x0001C941
			get;
			// Token: 0x060006C2 RID: 1730 RVA: 0x0001E749 File Offset: 0x0001C949
			set;
		}
	}
}
