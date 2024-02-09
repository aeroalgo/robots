using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000145 RID: 325
	public class MACDBase : IContextUses
	{
		// Token: 0x06000A06 RID: 2566 RVA: 0x0002A0EC File Offset: 0x000282EC
		protected IList<double> CalcMACD(IList<double> _src, int p1, int p2)
		{
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				p1.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.EMA(_src, p1));
			IList<double> data2 = this.Context.GetData("ema", new string[]
			{
				p2.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.EMA(_src, p2));
			List<double> list = new List<double>(data.Count);
			for (int i = 0; i < data.Count; i++)
			{
				list.Add(data[i] - data2[i]);
			}
			return list;
		}

		// Token: 0x1700034A RID: 842
		public IContext Context
		{
			// Token: 0x06000A07 RID: 2567 RVA: 0x0002A1ED File Offset: 0x000283ED
			get;
			// Token: 0x06000A08 RID: 2568 RVA: 0x0002A1F5 File Offset: 0x000283F5
			set;
		}
	}
}
