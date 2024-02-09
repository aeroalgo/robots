using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000BC RID: 188
	[HandlerCategory("vvTrade"), HandlerName("Удерживать сигнал N баров v2")]
	public class HoldSignal_v2 : IBoolConvertor, IOneSourceHandler, IBooleanReturns, IStreamHandler, IValuesHandler, IHandler, IBooleanInputs, IContextUses
	{
		// Token: 0x060006B2 RID: 1714 RVA: 0x0001E54C File Offset: 0x0001C74C
		public IList<bool> Execute(IList<bool> src)
		{
			int count = src.Count;
			bool[] array = new bool[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = false;
				if (src[i])
				{
					this._endbarnum = i + this.HoldBars - 1;
				}
				if (this._endbarnum > 0 && i > this._endbarnum - this.HoldBars && i <= this._endbarnum)
				{
					array[i] = true;
				}
			}
			return array;
		}

		// Token: 0x060006B3 RID: 1715 RVA: 0x0001E5B7 File Offset: 0x0001C7B7
		public bool Execute(bool src, int num)
		{
			if (src)
			{
				this._endbarnum = num + this.HoldBars - 1;
			}
			return (this._endbarnum > 0 && num > this._endbarnum - this.HoldBars && num <= this._endbarnum) || src;
		}

		// Token: 0x17000250 RID: 592
		public IContext Context
		{
			// Token: 0x060006B4 RID: 1716 RVA: 0x0001E5F1 File Offset: 0x0001C7F1
			get;
			// Token: 0x060006B5 RID: 1717 RVA: 0x0001E5F9 File Offset: 0x0001C7F9
			set;
		}

		// Token: 0x1700024F RID: 591
		[HandlerParameter(true, "5", Min = "1", Max = "30", Step = "1")]
		public int HoldBars
		{
			// Token: 0x060006B0 RID: 1712 RVA: 0x0001E53A File Offset: 0x0001C73A
			get;
			// Token: 0x060006B1 RID: 1713 RVA: 0x0001E542 File Offset: 0x0001C742
			set;
		}

		// Token: 0x0400025D RID: 605
		private int _endbarnum;
	}
}
