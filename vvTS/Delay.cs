using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000D1 RID: 209
	[HandlerCategory("vvTrade"), HandlerName("Задержка (баров)")]
	public class Delay : IBoolConvertor, IOneSourceHandler, IBooleanReturns, IStreamHandler, IValuesHandler, IHandler, IBooleanInputs
	{
		// Token: 0x060006FC RID: 1788 RVA: 0x0001F598 File Offset: 0x0001D798
		public IList<bool> Execute(IList<bool> src)
		{
			bool[] array = new bool[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i < this.Bars)
				{
					array[i] = false;
				}
				else if (src[i - this.Bars])
				{
					array[i] = true;
				}
				else
				{
					array[i] = false;
				}
			}
			return array;
		}

		// Token: 0x060006FD RID: 1789 RVA: 0x0001F5EC File Offset: 0x0001D7EC
		public bool Execute(bool src, int num)
		{
			return src;
		}

		// Token: 0x1700025B RID: 603
		[HandlerParameter(true, "1", Min = "0", Max = "5", Step = "1")]
		public int Bars
		{
			// Token: 0x060006FA RID: 1786 RVA: 0x0001F585 File Offset: 0x0001D785
			get;
			// Token: 0x060006FB RID: 1787 RVA: 0x0001F58D File Offset: 0x0001D78D
			set;
		}
	}
}
